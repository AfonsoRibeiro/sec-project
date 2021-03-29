//use structopt::StructOpt;
use color_eyre::eyre::Result;

use tonic::{transport::Server, Request, Response, Status};

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
            report : Some(Report {loc : "lol".to_string()}),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyLocationStorage::default();

    println!("LocationStorageServer listening on {}", addr);

    Server::builder()
        .add_service(LocationStorageServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
