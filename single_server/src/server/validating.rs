
use color_eyre::eyre::Result;

use std::{convert::TryFrom, sync::Arc};

use crate::storage::{Timeline,save_storage};

use tonic::{Request, Response, Status};

use protos::location_storage::location_storage_server::LocationStorage;
use protos::location_storage::{SubmitLocationReportRequest, SubmitLocationReportResponse,
    ObtainLocationReportRequest, ObtainLocationReportResponse};

use security::{key_management::ServerKeys, report::Report};
use security::proof::verify_proof;
use security::report::{decode_info, decode_report};
use security::status::{decode_loc_report, encode_loc_response};

use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;

pub struct MyLocationStorage {
    storage : Arc<Timeline>,
    server_keys : Arc<ServerKeys>,
    f_line : usize,
}

impl MyLocationStorage {
    pub fn new(storage : Arc<Timeline>, server_keys : Arc<ServerKeys>, f_line : usize) -> MyLocationStorage {
        MyLocationStorage {
            storage,
            server_keys,
            f_line
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

    fn check_valid_location_report(&self, req_idx : usize, report : &Report) -> bool {
        if req_idx != report.idx() { return false; }

        let (epoch, (pos_x, pos_y)) = (report.epoch(), report.loc());

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
}

#[tonic::async_trait]
impl LocationStorage for MyLocationStorage {
    async fn submit_location_report(
        &self,
        request: Request<SubmitLocationReportRequest>,
    ) -> Result<Response<SubmitLocationReportResponse>, Status> {
        let request = request.get_ref();

        let info = if let Ok(info) = decode_info(
            self.server_keys.private_key(),
            self.server_keys.public_key(),
            &request.report_info) {
            info
        } else {
            return Err(Status::permission_denied("Unhable to decrept sealed container"));
        };

        let client_sign_key = if let Some(ck) = self.server_keys.client_sign_key(info.idx()) {
            ck
        } else {
            return Err(Status::permission_denied(format!("Unable to find client {:} keys", info.idx())));
        };

        if !self.storage.valid_nonce(info.idx(), info.nonce()) {
            return Err(Status::already_exists("nonce already exists"));
        }

        let report = match decode_report(
            client_sign_key,
            info.key(),
            &request.report,
            &info.nonce(),
        ) {
            Ok(report) => {
                if !self.storage.add_nonce(info.idx(), info.nonce().clone()) {
                    return  Err(Status::permission_denied("nonce already exists"));
                }
                report
            }
            Err(_) => return  Err(Status::permission_denied("Unable to decrypt report"))
        };


        println!("Checking proofs");
        if self.check_valid_location_report(info.idx(), &report) {
            match self.storage.add_user_location_at_epoch(report.epoch(), report.loc(), info.idx(), request.report.clone()) {
                Ok(_) => {
                    if let Ok(_) = save_storage(self.storage.filename(), &self.storage).await {
                        let nonce = secretbox::gen_nonce();
                        Ok( Response::new(SubmitLocationReportResponse {
                            nonce : nonce.0.to_vec(),
                            ok : secretbox::seal(b"", &nonce, info.key()),
                        }))
                    } else {
                        Err(Status::aborted("Unable to permanently save information."))
                    }
                }
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

        let info = if let Ok(info) = decode_info(
            self.server_keys.private_key(),
            self.server_keys.public_key(),
            &request.user_info) {
            info
        } else {
            return Err(Status::permission_denied("Unhable to decrept sealed container"));
        };

        let client_sign_key = if let Some(ck) = self.server_keys.client_sign_key(info.idx()) {
            ck
        } else {
            return Err(Status::permission_denied(format!("Unable to find client {:} keys", info.idx())));
        };

        if !self.storage.valid_nonce(info.idx(), info.nonce()) {
            return Err(Status::already_exists("nonce already exists"));
        }

        let loc_req = match decode_loc_report(
            client_sign_key,
            info.key(),
            &request.user,
            info.nonce(),
        ) {
            Ok(location_request) => {
                if !self.storage.add_nonce(info.idx(), info.nonce().clone()) {
                    return  Err(Status::permission_denied("nonce already exists"));
                }
                location_request
            }
            Err(_) => return  Err(Status::permission_denied("Unable to decrypt report"))
        };
        match self.storage.get_user_location_at_epoch(loc_req.epoch(), loc_req.idx()) {
            Some((x,y )) =>  {
                let (location, nonce) = encode_loc_response(info.key(), x, y);
                Ok( Response::new(ObtainLocationReportResponse {
                    nonce : nonce.0.to_vec(),
                    location,
                }))
            }
            None => Err(Status::not_found(format!("User with id {:} not found at epoch {:}", loc_req.idx(), loc_req.epoch()))),
        }

    }
}