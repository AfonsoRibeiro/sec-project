use color_eyre::eyre::Result;

use tonic::{transport::Server, Request, Response, Status};

use protos::location_proof::location_proof_client::LocationProofClient;
use protos::location_proof::location_proof_server::{LocationProof, LocationProofServer};

use protos::location_proof::{RequestLocationProofRequest, RequestLocationProofResponse, Proof};

// As Server

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

// As Client

//let my_addr = format!("[::1]:6{:04}", opt.idx); // PORT: 6xxxx

async fn request_location_proof(idx : usize, epoch : usize, url : String) -> Result<()> {

    let mut client = LocationProofClient::connect(url).await?;

    let request = tonic::Request::new(RequestLocationProofRequest {
        idx: idx as u32,
        epoch: epoch as u32,
    });

    let response = client.request_location_proof(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
