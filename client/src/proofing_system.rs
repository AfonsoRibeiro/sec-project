use eyre::eyre;
use color_eyre::eyre::{Context, Result};

use std::sync::Arc;
use std::convert::TryFrom;

use grid::grid::Timeline;

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
    idx : usize,
    timeline : Arc<Timeline>
}

impl Proofer {
    fn new(idx : usize, timeline : Arc<Timeline>) -> Proofer {
        Proofer {
            idx,
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

        let result_req_idx = usize::try_from(request.get_ref().idx);
        if result_req_idx.is_err() || !self.timeline.is_point(result_req_idx.unwrap()) {
            return Err(Status::invalid_argument(format!("Not a valid id: {:}.", result_req_idx.unwrap())));
        }
        let req_idx = result_req_idx.unwrap();

        let result_req_epoch = usize::try_from(request.get_ref().epoch);
        if result_req_epoch.is_err() || self.timeline.epochs() <= result_req_epoch.unwrap() {
            return Err(Status::invalid_argument(format!("Not a valid epoch: {:}.", result_req_epoch.unwrap())));
        }
        let epoch = result_req_epoch.unwrap();

        match self.timeline.get_neighbours_at_epoch(self.idx, epoch) { // Maybe this verification is armful because it wont allow testing with byzantine users
            Some(neighbours) => {
                if neighbours.iter().any(|&i| i == req_idx) {
                    Ok(Response::new(RequestLocationProofResponse {
                        proof : Some (Proof {
                            idx_req : req_idx as u32,
                            loc : self.timeline.get_index_at_epoch(req_idx, epoch).unwrap() as u32,
                            epoch: epoch as u32,
                            idx_ass: self.idx as u32,
                        })

                    }))
                } else {
                    Err(Status::not_found("Can't prove that we are neighbours."))
                }
            }
            None => Err(Status::unknown("Will never happen."))
         }
    }
}

pub async fn start_proofer(idx : usize, timeline : Arc<Timeline>) -> Result<()> {
    let addr = get_address(idx).parse()?;
    let proofer = Proofer::new(idx, timeline);

    println!("LocationProofServer listening on {}", addr);

    Server::builder()
        .add_service(LocationProofServer::new(proofer))
        .serve(addr)
        .await?;

    Ok(())
}

// As Client

//let my_addr = format!("[::1]:6{:04}", opt.idx); // PORT: 6xxxx

pub async fn request_location_proof(idx : usize, epoch : usize, id_dest : usize) -> Result<Proof> {

    let mut client = LocationProofClient::connect(get_url(id_dest)).await.wrap_err_with(
        || format!("Failed to connect to client with id: {:}.", id_dest)
    )?;

    let request = tonic::Request::new(RequestLocationProofRequest {
        idx: idx as u32,
        epoch: epoch as u32,
    });

    match client.request_location_proof(request).await {
        Ok(response) => {
            match &response.get_ref().proof {
                Some(_) => { Ok(Proof::default()) }
                None => { Err(eyre!("Something failed."))  }
            }
        }
        Err(status) => Err(eyre!("RequestLocationProof failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    }
}