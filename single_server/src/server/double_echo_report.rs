use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}};

use futures::stream::{FuturesUnordered, StreamExt};
use futures::select;

use async_recursion::async_recursion;

use dashmap::DashMap;
use eyre::eyre;
use color_eyre::eyre::Result;
use sodiumoxide::crypto::{box_, secretbox, sign};
use tonic::{Request, Response, Status, transport::Uri};
use security::{double_echo::{self, Write, success_echo, decode_echo_info, decode_echo_request}, key_management::{ServerKeys, ServerPublicKey}, proof::verify_proof, report::{Report, verify_report}};
use protos::double_echo_broadcast::{EchoWriteRequest, EchoWriteResponse, double_echo_broadcast_client::DoubleEchoBroadcastClient, double_echo_broadcast_server::{DoubleEchoBroadcast}};

use crate::storage::{Timeline, save_storage};

struct Logic {
    n_servers : usize,
    echos  : DashMap<usize, DashMap<usize, Vec< Vec<u8>> > >, // client id -> epoch -> server id -> m
    readys : DashMap<usize, DashMap<usize, Vec< Vec<u8>> > >, // client id -> epoch -> server id -> m
    sent_echo  : RwLock<HashMap<usize, HashSet<usize>>>, // client id -> epoch
    sent_ready : RwLock<HashMap<usize, HashSet<usize>>>, // client id -> epoch
    delivered  : RwLock<HashMap<usize, HashSet<usize>>>, // client id -> epoch
}

impl Logic {
    fn new(n_servers : usize) -> Logic {
        Logic {
            n_servers,
            echos  : DashMap::new(),
            readys : DashMap::new(),
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
        match delivered.get_mut(&client_id) {
            Some(epoch_delivered) => epoch_delivered.insert(epoch),
            None => {
                let mut epoch_delivered = HashSet::new();
                epoch_delivered.insert(epoch);
                delivered.insert(client_id, epoch_delivered);
                true
            }
        }
    }
}

pub struct DoubleEcho {
    server_id : usize,
    server_urls : Vec<(usize, Uri)>,
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
        server_urls : Vec<(usize, Uri)>,
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
        epoch : usize,
        report : Report,
    ) -> Result<()> {

        // Check message doesnt exist yet (If so alredy checked)

        self.check_valid_location_report(client_id, &report);

        if self.logic.start_echo(client_id, epoch) {
            self.echo_fase(message, client_id, epoch).await;
        }

        // TODO wait for delivery done

        Ok(())
    }

    async fn echo_fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
        epoch : usize,
    ) -> Result<()> {

        let echo_write = Write::new_echo(message.clone(), client_id);

        if self.logic.start_echo(client_id, epoch) {
            self.fase(message, client_id, epoch, &echo_write, HashSet::new()).await
        } else {
            Ok(())
        }

    }

    async fn ready_fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
        epoch : usize,
    ) -> Result<()> {

        let echo_ready = Write::new_ready(message.clone(), client_id);

        if self.logic.start_ready(client_id, epoch) {
            self.fase(message, client_id, epoch, &echo_ready, HashSet::new()).await
        } else {
            Ok(())
        }
    }

    #[async_recursion]
    async fn fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
        epoch : usize,
        write : &Write,
        mut ack : HashSet<usize>,
    ) -> Result<()> {
        let mut responses : FuturesUnordered<_> =
            self.server_urls.iter().filter(
                |(server_id, _)|    !ack.contains(server_id)
            ).map(
                |(server_id, url)|
                    echo(
                        url,
                        self.server_id,
                        write,
                        self.server_keys.sign_key(),
                        self.server_pkeys.public_key(*server_id),
                    )
            ).collect();

        loop {
            select! {
                res = responses.select_next_some() => {
                    if let Ok(server_id) = res {
                        ack.insert(server_id);
                    }

                    if ack.len() > self.necessary_res {
                        break ;
                    }
                }
                complete => break,
            }
        }
        if ack.len() > self.necessary_res {
            Ok(())
        } else {
            self.fase(message, client_id, epoch, write, ack).await
        }
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

/*
CLIENT
*/

pub async fn echo(
    url : &Uri,
    server_id : usize,
    write : &double_echo::Write,
    sign_key : &sign::SecretKey,
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
                Ok(server_id)
            } else {
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

        let (write, message) = match decode_echo_request(
            self.echo.server_pkeys.public_sign_key(info.server_id),
            &info.key,
            &request.write,
            &info.nonce,
        ) {
            Ok(write_rep) => {
                write_rep
            }
            Err(_) => return  Err(Status::permission_denied("Unable to decrypt echo"))
        };

        let epoch = 0_usize; // FIX

        if write.is_echo() {

            // TODO add msg if not there

            // if > necessary_res
            if self.echo.logic.start_ready(write.client_id, epoch) {
                self.echo.ready_fase(&message, write.client_id, epoch);
            }

        } else { // READY

            // TODO add msg if not there

            // if > f_servers
            if self.echo.logic.start_ready(write.client_id, epoch) {
                self.echo.ready_fase(&message, write.client_id, epoch);
            }

            // if > necessary_res
                // Deliver
            if self.echo.logic.start_deliver(write.client_id, epoch) {
                self.echo.deliver(&message, write.client_id);
            }

        }

        let nonce = secretbox::gen_nonce();
        Ok( Response::new( EchoWriteResponse{
            nonce : nonce.0.to_vec(),
            ok : secretbox::seal(b"", &nonce, &info.key),
        }))
    }
}
