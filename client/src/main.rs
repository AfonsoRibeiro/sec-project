use structopt::StructOpt;
use color_eyre::eyre::Result;

use protos::location_storage::{SubmitLocationReportRequest, ObtainLocationReportRequest};
use protos::location_storage::location_storage_client::LocationStorageClient;

//use protos::location_proof::RequestLocationProofRequest;
use protos::location_proof::location_proof_client::LocationProofClient;


use tonic::{transport::Server, Request, Response, Status};

use protos::location_proof::location_proof_server::{LocationProof, LocationProofServer};
use protos::location_proof::{RequestLocationProofRequest, RequestLocationProofResponse, Proof};

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {
    #[structopt(name = "server", long, default_value="http://[::1]:50051")]
    server_url : String, // TODO
    #[structopt(long)]
    idx : usize,
    #[structopt(name = "grid", long, default_value="http://[::1]:50051")]
    grid_file : String
}

#[derive(Default)]
pub struct MyLocationProof {}


#[tonic::async_trait]
impl LocationProof for MyLocationProof {
    async fn request_location_proof(
        &self,
        request: Request<RequestLocationProofRequest>,
    ) -> Result<Response<RequestLocationProofResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        Ok(Response::new(RequestLocationProofResponse {proof : None}))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    // let mut client = LocationStorageClient::connect("http://[::1]:50051").await?;

    // let request = tonic::Request::new(ObtainLocationReportRequest {
    //     idx: 5,
    //     epoch: 6,
    // });

    // let response = client.obtain_location_report(request).await?;

    // println!("RESPONSE={:?}", response);

    let my_addr = format!("[::1]:6{:04}", opt.idx); // PORT: 6xxxx

    println!("{:}", my_addr);


    Ok(())

}
