
use color_eyre::eyre::Result;

use std::{convert::TryFrom, sync::Arc};

use crate::storage::Timeline;

use tonic::{Request, Response, Status};

use protos::location_storage::location_storage_server::{LocationStorage};
use protos::location_storage::{SubmitLocationReportRequest, SubmitLocationReportResponse,
    ObtainLocationReportRequest, ObtainLocationReportResponse, Report};

pub struct MyLocationStorage {
    storage : Arc<Timeline>,
}

impl MyLocationStorage {
    pub fn new(storage : Arc<Timeline>) -> MyLocationStorage {
        MyLocationStorage {
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

    fn parse_valid_location(&self, x : u32, y : u32) -> Result<(usize, usize), Status> {
        let res_x = usize::try_from(x);
        let res_y = usize::try_from(y);
        if res_x.is_err() || res_y.is_err() /*|| self.timeline.xs() <= result_req_x.unwrap()*/ {
            return Err(Status::invalid_argument(format!("Not a valid x or y: ({:}, {:}).", x, y)));
        }
        Ok((res_x.unwrap(), res_y.unwrap()))
    }
}

#[tonic::async_trait]
impl LocationStorage for MyLocationStorage {
    async fn submit_location_report(
        &self,
        request: Request<SubmitLocationReportRequest>,
    ) -> Result<Response<SubmitLocationReportResponse>, Status> {
        let request = request.get_ref();

        let (req_idx, epoch) =
            match (self.parse_valid_idx(request.idx), self.parse_valid_epoch(request.epoch)) {
                (Ok(idx), Ok(epoch)) => (idx, epoch),
                (Err(err), _) | (_, Err(err)) => return Err(err),
        };

        let (pos_x, pos_y) =  match self.parse_valid_location(request.pos_x, request.pos_y) {
            Ok(position) => position,
            Err(err) => return Err(err),
        };

        //self.storage.add_user_location_at_epoch(epoch, pos_x, pos_y, req_idx);

        Ok(Response::new(SubmitLocationReportResponse::default() ))
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

        let reply = ObtainLocationReportResponse {
            pos_x : 4,
            pos_y : 5,
        };

        Ok(Response::new(reply))
    }
}