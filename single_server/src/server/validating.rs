use color_eyre::eyre::Result;

use tonic::{Request, Response, Status};

use protos::location_storage::location_storage_server::{LocationStorage, LocationStorageServer};
use protos::location_storage::{SubmitLocationReportRequest, SubmitLocationReportResponse,
    ObtainLocationReportRequest, ObtainLocationReportResponse, Report};

#[derive(Default)]
pub struct MyLocationStorage {}

#[tonic::async_trait]
impl LocationStorage for MyLocationStorage {
    async fn submit_location_report(
        &self,
        request: Request<SubmitLocationReportRequest>,
    ) -> Result<Response<SubmitLocationReportResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        Ok(Response::new(SubmitLocationReportResponse {}))
    }

    async fn obtain_location_report(
        &self,
        request: Request<ObtainLocationReportRequest>,
    ) -> Result<Response<ObtainLocationReportResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = ObtainLocationReportResponse {
            report : Some(Report {proofs : vec![]}),
        };

        Ok(Response::new(reply))
    }
}