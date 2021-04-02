use color_eyre::eyre::Result;

use std::convert::TryFrom;

use tonic::{Request, Response, Status};

use protos::location_storage::location_storage_server::{LocationStorage, LocationStorageServer};
use protos::location_storage::{SubmitLocationReportRequest, SubmitLocationReportResponse,
    ObtainLocationReportRequest, ObtainLocationReportResponse, Report};

#[derive(Default)]
pub struct MyLocationStorage {}

impl MyLocationStorage {
    fn new() -> MyLocationStorage {
        MyLocationStorage {}
    }

    fn parse_valid_idx(&self, idx : u32) -> Result<usize, Status> {
        let idx = usize::try_from(idx);
        if idx.is_err() /*|| !self.timeline.is_point(idx.unwrap())*/ {
            return Err(Status::invalid_argument(format!("Not a valid id: {:}.", idx.unwrap())));
        }
        Ok(idx.unwrap())
    }

    fn parse_valid_epoch(&self, epoch : u32) -> Result<usize, Status> {
        let epoch = usize::try_from(epoch);
        if epoch.is_err() /*|| self.timeline.epochs() <= result_req_epoch.unwrap()*/ {
            return Err(Status::invalid_argument(format!("Not a valid epoch: {:}.", epoch.unwrap())));
        }
        Ok(epoch.unwrap())
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

        Ok(Response::new(SubmitLocationReportResponse {}))
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
            report : Some(Report {proofs : vec![]}),
        };

        Ok(Response::new(reply))
    }
}