use color_eyre::eyre::Result;

use std::{convert::TryFrom, sync::Arc};

use tonic::{Request, Response, Status};

use protos::location_master::location_master_server::LocationMaster;
use protos::location_master::{ObtainLocationReportRequest, ObtainLocationReportResponse,
    ObtainUsersAtLocationRequest, ObtainUsersAtLocationResponse};

use crate::storage::Timeline;

pub struct MyLocationMaster {
    storage : Arc<Timeline>,
}

impl MyLocationMaster {
    pub fn new(storage : Arc<Timeline>) -> MyLocationMaster {
        MyLocationMaster {
            storage,
        }
    }

    fn parse_valid_idx(&self, idx : u32) -> Result<usize, Status> {
        let res_idx = usize::try_from(idx);
        if res_idx.is_err() /*|| !self.timeline.is_point(idx.unwrap())*/ {
            return Err(Status::invalid_argument(format!("Not a valid id: {:}.", idx)));
        }
        Ok(res_idx.unwrap())
    }

    fn parse_valid_epoch(&self, epoch : u32) -> Result<usize, Status> {
        let res_epoch = usize::try_from(epoch);
        if res_epoch.is_err() /*|| self.timeline.epochs() <= result_req_epoch.unwrap()*/ {
            return Err(Status::invalid_argument(format!("Not a valid epoch: {:}.", epoch)));
        }
        Ok(res_epoch.unwrap())
    }

    fn parse_valid_pos(&self, x : u32, y : u32) -> Result<(usize, usize), Status> {
        let (res_x, res_y) = (usize::try_from(x), usize::try_from(y));
        if res_x.is_err() /* || check limits */ {
            return Err(Status::invalid_argument(format!("Not a valid x position: {:}.", x)));
        }
        if res_y.is_err() /* || check limits */ {
            return Err(Status::invalid_argument(format!("Not a valid y position: {:}.", y)));
        }
        Ok((res_x.unwrap(), res_y.unwrap()))
    }
}

#[tonic::async_trait]
impl LocationMaster for MyLocationMaster {
    async fn obtain_location_report(
        &self,
        request: Request<ObtainLocationReportRequest>,
    ) -> Result<Response<ObtainLocationReportResponse>, Status> {
        let request = request.get_ref();

        let (req_idx, epoch) =
            match (self.parse_valid_idx(request.idx), self.parse_valid_idx(request.epoch)) {
                (Ok(idx), Ok(epoch)) => (idx, epoch),
                (Err(err), _) | (_, Err(err)) => return Err(err),
        };

        match self.storage.get_user_location_at_epoch(epoch, req_idx) {
            Some((x,y )) => Ok(Response::new(ObtainLocationReportResponse { pos_x : x as u32, pos_y : y as u32,})),
            None => Err(Status::not_found(format!("User with id {:} not found at epoch {:}", req_idx, epoch))),
        } 
    }

    async fn obtain_users_at_location(
        &self,
        request : Request<ObtainUsersAtLocationRequest>
    ) ->Result<Response<ObtainUsersAtLocationResponse>, Status> {
        let request = request.get_ref();

        let ((x, y), epoch) =
            match (self.parse_valid_pos(request.pos_x, request.pos_y), self.parse_valid_epoch(request.epoch)) {
                (Ok(pos), Ok(epoch)) => (pos, epoch),
                (Err(err), _) | (_, Err(err)) => return Err(err),
        };

        match self.storage.get_users_at_epoch_at_location(epoch, x, y) {
            Some(users) => Ok(Response::new(ObtainUsersAtLocationResponse{ idxs : users.iter().map(|&idx| idx as u32).collect() })),
            None => Err(Status::not_found(format!("No users found at location ({:}, {:}) at epoch {:}", x, y, epoch))),
        } 
    }
}