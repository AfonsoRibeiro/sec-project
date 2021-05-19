use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}, time::Duration};

use futures::stream::{FuturesUnordered, StreamExt};
use futures::select;

use async_recursion::async_recursion;

use eyre::eyre;
use color_eyre::eyre::Result;
use sodiumoxide::crypto::{box_, secretbox, sign};
use tokio::time::sleep;
use tonic::{Request, Response, Status, transport::Uri};
use security::{double_echo::{self, Write, success_echo, decode_echo_info, decode_echo_request}, key_management::{ServerKeys, ServerPublicKey}, proof::verify_proof, report::{Report, verify_report}};
use protos::double_echo_broadcast::{EchoWriteRequest, EchoWriteResponse, double_echo_broadcast_client::DoubleEchoBroadcastClient, double_echo_broadcast_server::{DoubleEchoBroadcast}};

use crate::storage::{Timeline, save_storage};

struct Logic {
    n_servers : usize,
    echos  : RwLock<HashMap<usize, HashMap<Vec<u8>, HashSet<usize>>>>, // client id -> m -> server id
    readys : RwLock<HashMap<usize, HashMap<Vec<u8>, HashSet<usize>>>>, // client id -> m -> server id
    sent_echo  : RwLock<HashMap<usize, HashSet<usize>>>, // client id -> epoch
    sent_ready : RwLock<HashMap<usize, HashSet<usize>>>, // client id -> epoch
    delivered  : RwLock<HashMap<usize, HashSet<usize>>>, // client id -> epoch
}

impl Logic {
    fn new(n_servers : usize) -> Logic {
        Logic {
            n_servers,
            echos  : RwLock::new(HashMap::new()),
            readys : RwLock::new(HashMap::new()),
            sent_echo  : RwLock::new(HashMap::new()),
            sent_ready : RwLock::new(HashMap::new()),
            delivered  : RwLock::new(HashMap::new()),
        }
    }

    fn start_echo(&self, client_id : usize, epoch : usize) -> bool {
        { // TODO maybe add this to others
            let sent_echo = self.sent_echo.read().unwrap();
            if let Some(epoch_sent_echo) = sent_echo.get(&client_id) {
                if epoch_sent_echo.contains(&epoch) {
                    return false;
                }
            }
        }
        let mut sent_echo = self.sent_echo.write().unwrap();
        match sent_echo.get_mut(&client_id) {
            Some(epoch_sent_echo) => epoch_sent_echo.insert(epoch),
            None => {
                let mut epoch_sent_echo = HashSet::new();
                epoch_sent_echo.insert(epoch);
                sent_echo.insert(client_id, epoch_sent_echo);
                true
            }
        }
    }

    fn start_ready(&self, client_id : usize, epoch : usize) -> bool {
        let mut sent_ready = self.sent_ready.write().unwrap();
        match sent_ready.get_mut(&client_id) {
            Some(epoch_sent_ready) => epoch_sent_ready.insert(epoch),
            None => {
                let mut epoch_sent_ready = HashSet::new();
                epoch_sent_ready.insert(epoch);
                sent_ready.insert(client_id, epoch_sent_ready);
                true
            }
        }
    }

    fn start_deliver(&self, client_id : usize, epoch : usize) -> bool {
        let mut delivered = self.delivered.write().unwrap();
        let start = match delivered.get_mut(&client_id) {
            Some(epoch_delivered) => epoch_delivered.insert(epoch),
            None => {
                let mut epoch_delivered = HashSet::new();
                epoch_delivered.insert(epoch);
                delivered.insert(client_id, epoch_delivered);
                true
            }
        };

        if start {
            let mut client_echos = self.echos.write().unwrap();
            let mut client_readys = self.readys.write().unwrap();
            client_echos.insert(client_id, HashMap::new());
            client_readys.insert(client_id, HashMap::new());
        }
        start
    }

    fn has_been_delivered(&self, client_id : usize, epoch : usize) -> bool {
        match self.delivered.read().unwrap().get(&client_id) {
            Some(epochs_delivered) => epochs_delivered.contains(&epoch),
            None => false,
        }
    }

    fn has_echo_message(&self, client_id : usize, message : &Vec<u8>) -> bool{
        let client_msgs = self.echos.read().unwrap();
        match client_msgs.get(&client_id) {
            Some(msgs) => msgs.contains_key(message),
            None => false,
        }
    }

    fn add_server_to_echo_msg(&self, client_id : usize, server_id : usize, message : &Vec<u8>) -> usize {
        let mut client_msgs = self.echos.write().unwrap();
        match client_msgs.get_mut(&client_id) {
            Some(msgs) => {

                for set in msgs.values() {
                    if set.contains(&server_id) {
                        return set.len();
                    }
                }

                match msgs.get_mut(message) {
                    Some(set) => { set.insert(server_id); set.len() }
                    None => {
                        let mut y = HashSet::new();
                        y.insert(server_id);
                        msgs.insert(message.to_vec(), y);
                        1
                    }
                }

            }
            None => {
                let mut x = HashMap::new();
                let mut y = HashSet::new();
                y.insert(server_id);
                x.insert(message.to_vec(), y);
                client_msgs.insert(client_id, x);
                1
            },
        }
    }

    fn has_ready_message(&self, client_id : usize, message : &Vec<u8>) -> bool{
        let client_msgs = self.readys.read().unwrap();
        match client_msgs.get(&client_id) {
            Some(msgs) => msgs.contains_key(message),
            None => false,
        }
    }

    fn add_server_to_ready_msg(&self, client_id : usize, server_id : usize, message : &Vec<u8>) -> usize {
        let mut client_msgs = self.readys.write().unwrap();
        match client_msgs.get_mut(&client_id) {
            Some(msgs) => {

                for set in msgs.values() {
                    if set.contains(&server_id) {
                        return set.len();
                    }
                }

                match msgs.get_mut(message) {
                    Some(set) => { set.insert(server_id); set.len() }
                    None => {
                        let mut y = HashSet::new();
                        y.insert(server_id);
                        msgs.insert(message.to_vec(), y);
                        1
                    }
                }

            }
            None => {
                let mut x = HashMap::new();
                let mut y = HashSet::new();
                y.insert(server_id);
                x.insert(message.to_vec(), y);
                client_msgs.insert(client_id, x);
                1
            },
        }
    }
}

pub struct DoubleEcho {
    server_id : usize,
    server_urls : Arc<Vec<(usize, Uri)>>,
    necessary_res : usize,
    f_servers : usize,
    server_keys : Arc<ServerKeys>,
    server_pkeys : Arc<ServerPublicKey>,
    storage : Arc<Timeline>,
    f_line : usize,
    logic : Logic
}

impl DoubleEcho {
    pub fn new(
        server_id : usize,
        server_urls : Arc<Vec<(usize, Uri)>>,
        necessary_res : usize,
        f_servers : usize,
        server_keys : Arc<ServerKeys>,
        server_pkeys : Arc<ServerPublicKey>,
        f_line : usize,
        storage : Arc<Timeline>
) -> DoubleEcho {
        let n_servers = server_urls.len() + 1;

        DoubleEcho {
            server_id,
            server_urls,
            necessary_res,
            f_servers,
            server_keys,
            server_pkeys,
            storage,
            f_line,
            logic : Logic::new(n_servers),
        }
    }

    fn get_report_from_signed(
        &self,
        message : &Vec<u8>,
        client_id : usize,
    ) -> Result<Report> {
        if let Some(c_p_k) =  self.server_keys.client_sign_key(client_id) {
            match verify_report(c_p_k, message) {
                Ok(report) => Ok(report),
                Err(_) => return Err(eyre!("Could not verify report"))
            }
        } else {
            return Err(eyre!("user key not found"));
        }
    }

    fn check_valid_location_report(&self, req_idx : usize, report : &Report) -> bool { //signed report
        if req_idx != report.idx() { return false; }

        let (epoch, (pos_x, pos_y)) = (report.epoch(), report.loc());

        if !self.storage.valid_pos(pos_x, pos_y) {
            return false;
        }

        let ((lower_x, lower_y), (upper_x, upper_y)) = self.storage.valid_neighbour(pos_x, pos_y);
        let mut counter = 0;

        for (idx, proof) in report.proofs() {
            if let Some(sign_key) = self.server_keys.client_sign_key(*idx) {
                if let Ok(proof) = verify_proof(&sign_key, &proof) {
                    let (x, y)  = proof.loc_ass();
                    if lower_x <= x && x <= upper_x
                        && lower_y <= y && y <= upper_y
                        && epoch == proof.epoch()
                        && req_idx == proof.idx_req()
                        && *idx == proof.idx_ass() {
                        counter += 1;
                    }
                }
            }
            if counter > self.f_line {
                break;
            }
        }
        counter > self.f_line
    }

    fn correctly_ass_proofs(&self, report : &Report) -> Vec<(usize, usize, Vec<u8>)> { //signed report
        let mut proofs = vec![];
        for (idx, ass_proof) in report.proofs() {
            if let Some(sign_key) = self.server_keys.client_sign_key(*idx) {
                if let Ok(p) = verify_proof(&sign_key, &ass_proof) {
                    proofs.push((*idx, p.epoch(), ass_proof.clone()))
                }
            }
        }
        proofs
    }

    fn is_valid_server_id(&self, server_id : usize) -> bool {
        server_id < self.logic.n_servers
    }

    // LOGIC

    pub async fn confirm_write(
        &self,
        message : &Vec<u8>,
        client_id : usize,
        report : Report,
    ) -> Result<()> {

        if !self.check_valid_location_report(client_id, &report) {
            return Err(eyre!("Not a valid report"));
        }

        if self.logic.has_been_delivered(client_id, report.epoch()) {
            return Ok(());
        }

        if self.logic.start_echo(client_id, report.epoch()) {
            self.echo_fase(message, client_id, report.epoch());
        }

        // TODO wait for delivery done

        Ok(())
    }

    fn echo_fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
        epoch : usize,
    ) {
        let echo_write = Write::new_echo(message.clone(), client_id, epoch);

        self.logic.add_server_to_echo_msg(client_id, self.server_id, message);
        tokio::spawn(fase(
            self.server_id,
            echo_write,
            HashSet::new(),
            self.necessary_res,
            self.server_urls.clone(),
            self.server_keys.clone(),
            self.server_pkeys.clone(),
        ));

    }

    fn ready_fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
        epoch : usize,
    ) {
        let ready_write = Write::new_ready(message.clone(), client_id, epoch);

        self.logic.add_server_to_ready_msg(client_id, self.server_id, message);
        tokio::spawn(fase(
            self.server_id,
            ready_write,
            HashSet::new(),
            self.necessary_res,
            self.server_urls.clone(),
            self.server_keys.clone(),
            self.server_pkeys.clone(),
        ));
    }

    async fn deliver(
        &self,
        message : &Vec<u8>,
        client_id : usize,
    ) -> Result<()> {
        let report = self.get_report_from_signed(message, client_id)?;

        match self.storage.add_user_location_at_epoch(report.epoch(), report.loc(), client_id, message.clone()) {
            Ok(_) => self.storage.add_proofs(self.correctly_ass_proofs(&report)),
            Err(_) => return Err(eyre!("Unhable to add report")),
        }
        match save_storage(self.storage.filename(), &self.storage).await {
            Ok(_) => Ok(()),
            Err(_) => Err(eyre!("Unable to permanently save information.")),
        }
    }
}

#[async_recursion]
async fn fase(
    server_id : usize,
    write : Write,
    mut ack : HashSet<usize>,
    necessary_res : usize,
    server_urls : Arc<Vec<(usize, Uri)>>,
    server_keys : Arc<ServerKeys>,
    server_pkeys : Arc<ServerPublicKey>,
) -> Result<()> {
    let mut responses : FuturesUnordered<_> =
        server_urls.iter().filter(
            |(id, _)|    !ack.contains(id)
        ).map(
            |(id, url)|
                echo(
                    url,
                    server_id,
                    &write,
                    server_keys.sign_key(),
                    *id,
                    server_pkeys.public_key(*id),
                )
        ).collect();

    loop {
        select! {
            res = responses.select_next_some() => {

                if let Ok(id) = res {
                    ack.insert(id);
                }

                if ack.len() > necessary_res {
                    break ;
                }
            }
            complete => break,
        }
    }
    //println!("ack {:} | nec {:}", ack.len(), necessary_res);
    if ack.len() > necessary_res {
        Ok(())
    } else {
        sleep(Duration::from_millis(1000)).await;
        fase(
            server_id,
            write.clone(),
            ack,
            necessary_res,
            server_urls.clone(),
            server_keys.clone(),
            server_pkeys.clone(),
        ).await
    }
}

/*
CLIENT
*/

pub async fn echo(
    url : &Uri,
    server_id : usize,
    write : &double_echo::Write,
    sign_key : &sign::SecretKey,
    dest_id : usize,
    server_key : &box_::PublicKey,
) -> Result<usize> {
    let (info, write, key) = double_echo::encode_echo_request(sign_key, server_key, write, server_id);
    let mut client = DoubleEchoBroadcastClient::connect(url.clone()).await?;

    let request =
        tonic::Request::new( EchoWriteRequest{
            write,
            info,
        });

    match client.echo_write(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if success_echo(&key, &response.nonce, &response.ok) {
                Ok(dest_id)
            } else {
                println!("Failed echo");
                Err(eyre!("echo_write unable to validate server response"))
            }
        }
        Err(status) => {
            println!("Echo write failed with code {:?} and message {:?}.",
            status.code(), status.message());
            Err(eyre!("Echo write failed with code {:?} and message {:?}.",
                            status.code(), status.message()))
        }
    }
}

/*
SERVER
*/
pub struct MyDoubleEchoWrite {
    echo : Arc<DoubleEcho>,
}

impl MyDoubleEchoWrite {
    pub fn new(
        echo : Arc<DoubleEcho>,
    ) -> MyDoubleEchoWrite {

        MyDoubleEchoWrite {
            echo,
        }
    }
}

#[tonic::async_trait]
impl DoubleEchoBroadcast for MyDoubleEchoWrite {
    async fn echo_write(
        &self,
        request : Request<EchoWriteRequest>,
    ) ->  Result<Response<EchoWriteResponse>, Status> {
        let request = request.get_ref();

        let info = if let Ok(info) = decode_echo_info(
            self.echo.server_keys.private_key(),
            self.echo.server_keys.public_key(),
            &request.info) {
            info
        } else {
            return Err(Status::permission_denied("Unhable to decrypt sealed container"));
        };

        if !self.echo.is_valid_server_id(info.server_id) {
            return Err(Status::permission_denied(format!("Unable to find server {:} keys", info.server_id)));
        };

        let write = match decode_echo_request(
            self.echo.server_pkeys.public_sign_key(info.server_id),
            &info.key,
            &request.write,
            &info.nonce,
        ) {
            Ok(write) => {
                write
            }
            Err(_) => return Err(Status::permission_denied("Unable to decrypt echo"))
        };

        let message = &write.report;

        if write.is_echo() {

            if !self.echo.logic.has_echo_message(write.client_id, message) {
                match self.echo.get_report_from_signed(message, write.client_id) {
                    Ok(report) => {
                        if !self.echo.check_valid_location_report(write.client_id, &report)
                            && write.epoch == report.epoch() {
                            return Err(Status::aborted("Not a correct report"));
                        }
                        if self.echo.logic.has_been_delivered(write.client_id, write.epoch) {
                            let nonce = secretbox::gen_nonce();
                            return Ok( Response::new( EchoWriteResponse{
                                nonce : nonce.0.to_vec(),
                                ok : secretbox::seal(b"", &nonce, &info.key),
                            }));
                        }
                    }
                    Err(err) => return Err(Status::aborted(err.to_string())),
                }
            }

            if self.echo.logic.add_server_to_echo_msg(write.client_id, info.server_id, message) > self.echo.necessary_res {
                if self.echo.logic.start_ready(write.client_id, write.epoch) {
                    self.echo.ready_fase(message, write.client_id, write.epoch);
                }
            }

        } else { // READY

            if !self.echo.logic.has_ready_message(write.client_id, message) {
                match self.echo.get_report_from_signed(message, write.client_id) {
                    Ok(report) => {
                        if !self.echo.check_valid_location_report(write.client_id, &report)
                            && write.epoch == report.epoch() {
                            return Err(Status::aborted("Not a correct report"));
                        }
                        if self.echo.logic.has_been_delivered(write.client_id, write.epoch) {
                            let nonce = secretbox::gen_nonce();
                            return Ok( Response::new( EchoWriteResponse{
                                nonce : nonce.0.to_vec(),
                                ok : secretbox::seal(b"", &nonce, &info.key),
                            }));
                        }
                    }
                    Err(err) => return Err(Status::aborted(err.to_string())),
                }
            }

            let n = self.echo.logic.add_server_to_ready_msg(write.client_id, info.server_id, message);

            if n > self.echo.f_servers {
                if self.echo.logic.start_ready(write.client_id, write.epoch) {
                    self.echo.ready_fase(message, write.client_id, write.epoch);
                }
            }
            if n > self.echo.necessary_res {
                if self.echo.logic.start_deliver(write.client_id, write.epoch) {
                    let _x = self.echo.deliver(message, write.client_id).await;
                }
            }

        }

        let nonce = secretbox::gen_nonce();
        Ok( Response::new( EchoWriteResponse{
            nonce : nonce.0.to_vec(),
            ok : secretbox::seal(b"", &nonce, &info.key),
        }))
    }
}
