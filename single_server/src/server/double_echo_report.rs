use std::sync::Arc;

use dashmap::{DashMap, DashSet};
use eyre::eyre;
use color_eyre::eyre::Result;
use sodiumoxide::crypto::{box_, secretbox, sign};
use tonic::{Request, Response, Status, transport::Uri};
use security::{double_echo::{self, Write, success_echo, decode_echo_info, decode_echo_request}, key_management::{ServerKeys, ServerPublicKey}, proof::verify_proof, report::{Report, verify_report}};
use protos::double_echo_broadcast::{EchoWriteRequest, EchoWriteResponse, double_echo_broadcast_client::DoubleEchoBroadcastClient, double_echo_broadcast_server::{DoubleEchoBroadcast}};

use crate::storage::Timeline;

struct Logic {
    n_servers : usize,
    echos : DashMap<usize,  Vec< Vec<u8> > >, // client id -> server id -> m
    readys : DashMap<usize, Vec< Vec<u8> > >, // client id -> server id -> m
    sent_echo : DashSet<usize>, // client id
    sent_ready : DashSet<usize>, // client id
    delivered : DashSet<usize>, // client id
}

impl Logic {
    fn new(n_servers : usize) -> Logic {
        Logic {
            n_servers,
            echos : DashMap::new(),
            readys : DashMap::new(),
            sent_echo : DashSet::new(),
            sent_ready : DashSet::new(),
            delivered : DashSet::new(),
        }
    }
}

pub struct DoubleEcho {
    server_id : usize,
    server_urls : Arc<Vec<Uri>>,
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
        server_urls : Arc<Vec<Uri>>,
        necessary_res : usize,
        f_servers : usize,
        server_keys : Arc<ServerKeys>,
        server_pkeys : Arc<ServerPublicKey>,
        f_line : usize,
        storage : Arc<Timeline>
) -> DoubleEcho {
        let n_servers = server_urls.len();

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

    fn is_valid_server_id(&self, server_id : usize) -> bool {
        server_id < self.logic.n_servers
    }

    pub async fn confirm_write(
        &self,
        message : &Vec<u8>,
        client_id : usize,
    ) -> Result<()> {
        if self.logic.sent_echo.insert(client_id) {
            self.echo_fase(message, client_id).await;
        }
        // TODO wait for delivery done
        Ok(())
    }

    pub async fn echo_fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
    ) -> Result<()> {

        let echo_write = Write::new_echo(message.clone(), client_id);

        if self.logic.sent_echo.insert(client_id) {
            for (server_id, url) in self.server_urls.iter().enumerate() {
                let _res = echo(
                    url,
                    self.server_id,
                    &echo_write,
                    self.server_keys.sign_key(),
                    self.server_pkeys.public_key(server_id),
                );
            }
        }

        Ok(())
    }

    pub async fn ready_fase(
        &self,
        message : &Vec<u8>,
        client_id : usize,
    ) -> Result<()> {

        let echo_ready = Write::new_ready(message.clone(), client_id);

        if self.logic.sent_ready.insert(client_id) {
            for (server_id, url) in self.server_urls.iter().enumerate() {
                let _res2 = echo(
                    url,
                    self.server_id,
                    &echo_ready,
                    self.server_keys.sign_key(),
                    self.server_pkeys.public_key(server_id),
                ).await;
            }
        }

        Ok(())
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
) -> Result<()> {
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
                Ok(())
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

        let (write, signed_rep) = match decode_echo_request(
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

        if let Some(c_p_k) =  self.echo.server_keys.client_sign_key(write.client_id) {
            match verify_report(c_p_k, &signed_rep) {
                Ok(report) => if !self.echo.check_valid_location_report(write.client_id, &report) {
                        return Err(Status::cancelled("Could not verify report"));
                    },
                Err(_) => return Err(Status::cancelled("Could not verify report"))
            }
        } else {
            return Err(Status::permission_denied("user key not found"));
        }

        if write.is_echo() {

            // TODO add msg if not there

            // if > necessary_res
            if self.echo.logic.sent_ready.insert(write.client_id) {
                self.echo.ready_fase(&signed_rep, write.client_id);
            }

        } else { // READY

            // TODO add msg if not there

            // if > f_servers
            if self.echo.logic.sent_ready.insert(write.client_id) {
                self.echo.ready_fase(&signed_rep, write.client_id);
            }

            // if > necessary_res
                // Deliver
                self.echo.logic.delivered.insert(write.client_id);

        }

        let nonce = secretbox::gen_nonce();
        Ok( Response::new( EchoWriteResponse{
            nonce : nonce.0.to_vec(),
            ok : secretbox::seal(b"", &nonce, &info.key),
        }))
    }
}
