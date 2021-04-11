
use color_eyre::eyre::Result;
use sodiumoxide::crypto::box_;

use std::{convert::TryFrom, sync::Arc};

use crate::storage::Timeline;

use tonic::{Request, Response, Status};

use protos::location_storage::location_storage_server::LocationStorage;
use protos::location_storage::{SubmitLocationReportRequest, SubmitLocationReportResponse,
    ObtainLocationReportRequest, ObtainLocationReportResponse};

use security::{key_management::ServerKeys, report::Report};
use security::proof::verify_proof;
use security::report::decode_report;

pub struct MyLocationStorage {
    storage : Arc<Timeline>,
    server_keys : Arc<ServerKeys>,
}

impl MyLocationStorage {
    pub fn new(storage : Arc<Timeline>, server_keys : Arc<ServerKeys>) -> MyLocationStorage {
        MyLocationStorage {
            storage,
            server_keys,
        }
    }

    fn parse_valid_idx(&self, idx : u64) -> Result<usize, Status> {
        let res_idx = usize::try_from(idx);
        if res_idx.is_err() {
            return Err(Status::invalid_argument(format!("Not a valid id: {:}.", idx)));
        }
        Ok(res_idx.unwrap())
    }

    fn parse_valid_epoch(&self, epoch : u64) -> Result<usize, Status> {
        let res_epoch = usize::try_from(epoch);
        if res_epoch.is_err() {
            return Err(Status::invalid_argument(format!("Not a valid epoch: {:}.", epoch)));
        }
        Ok(res_epoch.unwrap())
    }

    fn parse_valid_location(&self, x : u64, y : u64) -> Result<(usize, usize), Status> {
        let res_x = usize::try_from(x);
        let res_y = usize::try_from(y);
        if res_x.is_err() || res_y.is_err() || !self.storage.valid_pos(res_x.unwrap(), res_y.unwrap()) {
            return Err(Status::invalid_argument(format!("Not a valid x or y: ({:}, {:}).", x, y)));
        }
        Ok((res_x.unwrap(), res_y.unwrap()))
    }

    fn parse_valid_nonce(&self, nonce : &[u8]) -> Result<box_::Nonce, Status> { // TODO: check if first nonce for user
        match box_::Nonce::from_slice(nonce) {
            Some(nonce) => Ok(nonce),
            None => return Err(Status::invalid_argument("Not a valid nonce")),
        }
    }

    fn check_valid_location_report(&self, req_idx : usize, report : &Report) -> bool {
        if req_idx != report.idx() { return false; }

        let (epoch, (pos_x, pos_y)) = (report.epoch(), report.loc());


        let f_line : usize = 1; // TODO

        let ((lower_x, lower_y), (upper_x, upper_y)) = self.storage.valid_neighbour(pos_x, pos_y);
        let mut counter = 0;

        for (idx, proof) in report.proofs() {
            if let Some(sign_key) = self.server_keys.client_sign_keys(*idx) {
                if let Ok(proof) = verify_proof(&sign_key, &proof) {
                    let (x, y)  = proof.loc_ass();
                    if lower_x <= x && x <= upper_x
                        && lower_y <= y && y <= upper_y
                        && epoch == proof.epoch()
                        && req_idx == proof.idx_req()
                        && *idx == proof.idx_ass() {
                        println!("Valid proof");
                        counter += 1;
                    }
                }
            }
            if counter > f_line {
                break;
            }
        }
        counter > f_line
    }
}

#[tonic::async_trait]
impl LocationStorage for MyLocationStorage {
    async fn submit_location_report(
        &self,
        request: Request<SubmitLocationReportRequest>,
    ) -> Result<Response<SubmitLocationReportResponse>, Status> {
        let request = request.get_ref();
        println!("Client {:} sending a location report", request.idx);
        let req_idx =
            match self.parse_valid_idx(request.idx) {
                Ok(idx) => idx,
                Err(err) => return Err(err),
        };

        let (client_key, client_sign_key) = if let Some(ck) = self.server_keys.all_client_keys(req_idx) {
            ck
        } else {
            return Err(Status::permission_denied(format!("Unable to find client {:} keys", req_idx)));
        };

        let nonce = match self.parse_valid_nonce(&request.nonce) {
            Ok(nonce) => nonce,
            Err(err) => return Err(err),
        };

        let report = match decode_report(
            client_sign_key,
            self.server_keys.private_key(),
            client_key,
            &request.report,
            nonce
        ) {
            Ok(report) => report,
            Err(_) => return  Err(Status::permission_denied("Unable to decrypt report"))
        };

        println!("Checking proofs");
        if self.check_valid_location_report(req_idx, &report){
            match self.storage.add_user_location_at_epoch(report.epoch(), report.loc(), req_idx) {
                Ok(_) => Ok(Response::new(SubmitLocationReportResponse::default() )),
                Err(_) => Err(Status::permission_denied("Permission denied!!")),
            }
         } else {
            println!("Failed");
            Err(Status::permission_denied("Report not valid!!"))
        }
    }

    async fn obtain_location_report(
        &self,
        request: Request<ObtainLocationReportRequest>,
    ) -> Result<Response<ObtainLocationReportResponse>, Status> {
        let request = request.get_ref();

        let (req_idx, epoch) =
            match (self.parse_valid_idx(request.idx), self.parse_valid_epoch(request.epoch)) {
                (Ok(idx), Ok(epoch)) => (idx, epoch),
                (Err(err), _) | (_, Err(err)) => return Err(err),
        };
        match self.storage.get_user_location_at_epoch(epoch, req_idx) {
            Some((x,y )) => Ok(Response::new(ObtainLocationReportResponse { pos_x : x as u64, pos_y : y as u64,})),
            None => Err(Status::not_found(format!("User with id {:} not found at epoch {:}", req_idx, epoch))),
        }
    }
}