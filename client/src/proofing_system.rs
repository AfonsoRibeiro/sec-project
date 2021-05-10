use eyre::eyre;
use color_eyre::eyre::{Context, Result};

use std::sync::Arc;
use std::convert::TryFrom;

use grid::grid::Timeline;

use futures::stream::{FuturesUnordered, StreamExt};
use futures::select;

use tonic::{transport::Server, Request, Response, Status};

use protos::location_proof::location_proof_client::LocationProofClient;
use protos::location_proof::location_proof_server::{LocationProof, LocationProofServer};

use protos::location_proof::{RequestLocationProofRequest, RequestLocationProofResponse};
use security::proof::{self, sign_proof};

use sodiumoxide::crypto::sign;

fn get_address(idx : usize) -> String {
    format!("[::1]:6{:04}", idx)
}

fn get_url(idx : usize) -> String {
    format!("http://{:}", get_address(idx))
}

// As Server
struct Proofer {
    idx : usize,
    timeline : Arc<Timeline>,
    sign_key : sign::SecretKey,
}

impl Proofer {
    fn new(idx : usize, timeline : Arc<Timeline>, sign_key : sign::SecretKey) -> Proofer {
        Proofer {
            idx,
            timeline,
            sign_key,
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

        // Maybe this verification is armful because it wont allow testing with byzantine users
        // And the request can only be recieved by a neighbour
        match self.timeline.get_neighbours_at_epoch(self.idx, epoch) {
            Some(neighbours) => {
                if neighbours.iter().any(|&i| i == req_idx) {
                    let (x, y) = self.timeline.get_location_at_epoch(self.idx, epoch).unwrap();
                    Ok(Response::new(RequestLocationProofResponse {
                        proof : sign_proof(&self.sign_key, proof::Proof::new(
                                epoch,
                                req_idx,
                                self.idx,
                                (x, y),
                            )),
                        idx_ass : self.idx as u64,

                    }))
                } else {
                    Err(Status::not_found("Can't prove that we are neighbours."))
                }
            }
            None => Err(Status::unknown("Will never happen."))
         }
    }
}

pub async fn start_proofer(idx : usize, timeline : Arc<Timeline>, sign_key : sign::SecretKey) -> Result<()> {
    let addr = get_address(idx).parse()?;
    let proofer = Proofer::new(idx, timeline, sign_key);

    println!("LocationProofServer listening on {}\n", addr);

    Server::builder()
        .add_service(LocationProofServer::new(proofer))
        .serve(addr)
        .await?;

    Ok(())
}

// As Client

pub async fn request_location_proof(idx : usize, epoch : usize, id_dest : usize) -> Result<(Vec<u8>, u64)> {

    let mut client = LocationProofClient::connect(get_url(id_dest)).await.wrap_err_with(
        || format!("Failed to connect to client with id: {:}.", id_dest)
    )?;

    let request = tonic::Request::new(RequestLocationProofRequest {
        idx: idx as u64,
        epoch: epoch as u64,
    });
    match client.request_location_proof(request).await {

        Ok(response) => {
            Ok((response.get_ref().proof.clone(), response.get_ref().idx_ass))
        }
        Err(status) => Err(eyre!("RequestLocationProof failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    }
}

pub async fn get_proofs(timeline : Arc<Timeline>, idx : usize, epoch : usize) -> (Vec<Vec<u8>>, Vec<usize>) {

    let nec_proofs = timeline.f_line * 2;

    let neighbours = match timeline.get_neighbours_at_epoch(idx, epoch) {
        Some(neighbours) => neighbours,
        None => panic!("Should nerver occour : Idx {:} from args doens't exist in grid.", idx),
    };

    let mut responses : FuturesUnordered<_> = neighbours.iter().map(
        |&id_dest| request_location_proof(idx, epoch, id_dest)
    ).collect();

    let mut report : Vec<Vec<u8>> = Vec::with_capacity(nec_proofs + 1);
    let mut idx : Vec<usize> = Vec::with_capacity(nec_proofs + 1);  // Number of proofs needed

    loop {
        select! {
            res = responses.select_next_some() => {
                if let Ok((proof, idx_ass)) = res {
                    if let Ok(x) = usize::try_from(idx_ass) {
                        report.push(proof);
                        idx.push(x);
                    }
                }

                if report.len() > nec_proofs {
                    break ;
                }
            }
            complete => break,
        }
    }
    (report, idx)
}
