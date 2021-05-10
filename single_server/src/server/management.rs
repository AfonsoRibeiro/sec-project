use color_eyre::eyre::Result;
use security::key_management::ServerKeys;

use std::sync::Arc;

use tonic::{Request, Response, Status};

use protos::location_master::location_master_server::LocationMaster;
use protos::location_master::{ObtainLocationReportRequest, ObtainLocationReportResponse,
    ObtainUsersAtLocationRequest, ObtainUsersAtLocationResponse};

use crate::storage::Timeline;

use security::report::decode_info;
use security::status::{decode_loc_report, encode_loc_response, decode_users_at_loc_report, encode_users_at_loc_response};

pub struct MyLocationMaster {
    storage : Arc<Timeline>,
    server_keys : Arc<ServerKeys>,
}

impl MyLocationMaster {
    pub fn new(storage : Arc<Timeline>, server_keys : Arc<ServerKeys>,) -> MyLocationMaster {
        MyLocationMaster {
            storage,
            server_keys,
        }
    }
}

#[tonic::async_trait]
impl LocationMaster for MyLocationMaster {
    async fn obtain_location_report(
        &self,
        request: Request<ObtainLocationReportRequest>,
    ) -> Result<Response<ObtainLocationReportResponse>, Status> {
        let request = request.get_ref();

        let info = if let Ok(info) = decode_info(
            self.server_keys.private_key(),
            self.server_keys.public_key(),
            &request.info) {
            info
        } else {
            return Err(Status::permission_denied("Unhable to decrept sealed container"));
        };

        if !self.storage.valid_ha_nonce(info.nonce()) {
            return Err(Status::already_exists("nonce already exists"));
        }

        let loc_req = match decode_loc_report(
            self.server_keys.ha_public_key(),
            info.key(),
            &request.user,
            info.nonce(),
        ) {
            Ok(location_request) => {
                if !self.storage.add_ha_nonce(info.nonce().clone()) {
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

    async fn obtain_users_at_location(
        &self,
        request : Request<ObtainUsersAtLocationRequest>
    ) ->Result<Response<ObtainUsersAtLocationResponse>, Status> {

        let request = request.get_ref();

        let info = if let Ok(info) = decode_info(
            self.server_keys.private_key(),
            self.server_keys.public_key(),
            &request.info) {
            info
        } else {
            return Err(Status::permission_denied("Unhable to decrept sealed container"));
        };

        if !self.storage.valid_ha_nonce(info.nonce()) {
            return Err(Status::already_exists("nonce already exists"));
        }

        let loc_req = match decode_users_at_loc_report(
            self.server_keys.ha_public_key(),
            info.key(),
            &request.place,
            info.nonce(),
        ) {
            Ok(location_request) => {
                if !self.storage.add_ha_nonce(info.nonce().clone()) {
                    return  Err(Status::permission_denied("nonce already exists"));
                }
                location_request
            }
            Err(_) => return  Err(Status::permission_denied("Unable to decrypt report"))
        };
        match self.storage.get_users_at_epoch_at_location(loc_req.epoch(), loc_req.pos()) {
            Some(idxs) =>  {
                let (idxs, nonce) = encode_users_at_loc_response(info.key(), idxs);
                Ok( Response::new(ObtainUsersAtLocationResponse {
                    nonce : nonce.0.to_vec(),
                    idxs,
                }))
            }
            None => Err(Status::not_found(format!("Location {:?} at epoch {:} not found.", loc_req.pos(), loc_req.epoch()))),
        }
    }
}