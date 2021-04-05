
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
}

#[tonic::async_trait]
impl LocationStorage for MyLocationStorage {
    async fn submit_location_report(
        &self,
        request: Request<SubmitLocationReportRequest>,
    ) -> Result<Response<SubmitLocationReportResponse>, Status> {
        let request = request.get_ref();

        let (req_idx, epoch) =
            match (self.parse_valid_idx(request.idx), self.parse_valid_idx(request.epoch)) {
                (Ok(idx), Ok(epoch)) => (idx, epoch),
                (Err(err), _) | (_, Err(err)) => return Err(err),
        };

        

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