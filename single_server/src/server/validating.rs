
use color_eyre::eyre::Result;

use std::{sync::Arc, usize};

use crate::storage::Timeline;

use tonic::{Request, Response, Status};

use protos::location_storage::{RequestMyProofsRequest, RequestMyProofsResponse, location_storage_server::LocationStorage};
use protos::location_storage::{SubmitLocationReportRequest, SubmitLocationReportResponse,
    ObtainLocationReportRequest, ObtainLocationReportResponse};

use security::key_management::ServerKeys;
use security::report::{decode_info, decode_report};
use security::status::{decode_loc_report, encode_loc_response, decode_my_proofs_request, encode_my_proofs_response};

use sodiumoxide::crypto::secretbox;

use super::double_echo_report::DoubleEcho;

pub struct MyLocationStorage {
    storage : Arc<Timeline>,
    server_keys : Arc<ServerKeys>,
    f_line : usize,
    echo : Arc<DoubleEcho>,
}

impl MyLocationStorage {
    pub fn new(
        storage : Arc<Timeline>,
        server_keys : Arc<ServerKeys>,
        f_line : usize,
        echo : Arc<DoubleEcho>,
    ) -> MyLocationStorage {

        MyLocationStorage {
            storage,
            server_keys,
            f_line,
            echo
        }
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
            return Err(Status::permission_denied("Unhable to decrypt sealed container"));
        };

        let client_sign_key = if let Some(ck) = self.server_keys.client_sign_key(info.idx()) {
            ck
        } else {
            return Err(Status::permission_denied(format!("Unable to find client {:} keys", info.idx())));
        };

        if !self.storage.valid_nonce(info.idx(), info.nonce()) {
            return Err(Status::already_exists("nonce already exists"));
        }

        let (report, signed_rep) = match decode_report(
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

        // TODO CHECK NOT ALREADY SENTECHO = FALSE

        if !self.storage.report_not_submitted_at_epoch(report.epoch(), info.idx()) {
            let nonce = secretbox::gen_nonce();
            return Ok(Response::new(SubmitLocationReportResponse {
                nonce : nonce.0.to_vec(),
                ok : secretbox::seal(b"", &nonce, info.key()),
            }));
        }

        match self.echo.confirm_write(&signed_rep, report.idx(), report.epoch(), report).await {
            Ok(_) => {
                let nonce = secretbox::gen_nonce();
                Ok( Response::new(SubmitLocationReportResponse {
                    nonce : nonce.0.to_vec(),
                    ok : secretbox::seal(b"", &nonce, info.key()),
                }))
            }
            Err(err) => Err(Status::aborted(err.to_string())),
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
            return Err(Status::permission_denied("Unhable to decript sealed container"));
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
        match self.storage.get_user_report_at_epoch(loc_req.epoch(), loc_req.idx()) {
            Some(report) =>  {
                let (location, nonce) = encode_loc_response(info.key(), report.clone());
                Ok( Response::new(ObtainLocationReportResponse {
                    nonce : nonce.0.to_vec(),
                    location,
                }))
            }
            None => Err(Status::not_found(format!("User with id {:} not found at epoch {:}", loc_req.idx(), loc_req.epoch()))),
        }

    }

    async fn request_my_proofs(
        &self,
        request : Request<RequestMyProofsRequest>,
    ) -> Result<Response<RequestMyProofsResponse>, Status> {
        let request = request.get_ref();

        let info = if let Ok(info) = decode_info(
            self.server_keys.private_key(),
            self.server_keys.public_key(),
            &request.user_info) {
            info
        } else {
            return Err(Status::permission_denied("Unhable to decript sealed container"));
        };

        let client_sign_key = if let Some(ck) = self.server_keys.client_sign_key(info.idx()) {
            ck
        } else {
            return Err(Status::permission_denied(format!("Unable to find client {:} keys", info.idx())));
        };

        if !self.storage.valid_nonce(info.idx(), info.nonce()) {
            return Err(Status::already_exists("nonce already exists"));
        }

        let proofs_req = match decode_my_proofs_request(
            client_sign_key,
            info.key(),
            &request.epochs,
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

        let (proofs, nonce) = encode_my_proofs_response(info.key(), self.storage.get_proofs(info.idx(), &proofs_req.epochs));

        Ok( Response::new( RequestMyProofsResponse {
            nonce : nonce.0.to_vec(),
            proofs,
        }))
    }
}