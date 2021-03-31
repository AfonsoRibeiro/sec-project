use color_eyre::eyre::{Context, Result};

use grid::grid::Timeline;

use std::sync::Arc;

use tonic::{transport::Server, Request, Response, Status};

use protos::location_proof::location_proof_client::LocationProofClient;
use protos::location_proof::location_proof_server::{LocationProof, LocationProofServer};

use protos::location_proof::{RequestLocationProofRequest, RequestLocationProofResponse, Proof};

fn get_address(idx : usize) -> String {
    format!("[::1]:6{:04}", idx)
}

fn get_url(idx : usize) -> String {
    format!("http://{:}", get_address(idx))
}

// As Server
struct Proofer {
    timeline : Arc<Timeline>,
}

impl Proofer {
    fn new(timeline : Arc<Timeline>) -> Proofer {
        Proofer {
            timeline,
        }
    }
}

#[tonic::async_trait]
impl LocationProof for Proofer {
    async fn request_location_proof(
        &self,
        request: Request<RequestLocationProofRequest>,
    ) -> Result<Response<RequestLocationProofResponse>, Status> {



        Ok(Response::new(RequestLocationProofResponse {proof : None}))
    }
}

pub async fn start_proofer(idx : usize, timeline : Arc<Timeline>) -> Result<()> {
    let addr = get_address(idx).parse()?;
    let proofer = Proofer::new(timeline);

    println!("LocationProofServer listening on {}", addr);

    Server::builder()
        .add_service(LocationProofServer::new(proofer))
        .serve(addr)
        .await?;

    Ok(())
}

// As Client

//let my_addr = format!("[::1]:6{:04}", opt.idx); // PORT: 6xxxx

pub async fn request_location_proof(idx : usize, epoch : usize, id_dest : usize) -> Result<()> {

    let mut client = LocationProofClient::connect(get_url(id_dest)).await.wrap_err_with(
        || format!("Failed to connect to client with id: {:}.", id_dest)
    )?;

    let request = tonic::Request::new(RequestLocationProofRequest {
        idx: idx as u32,
        epoch: epoch as u32,
    });

    let response = client.request_location_proof(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
