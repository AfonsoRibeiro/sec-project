use std::borrow::BorrowMut;

use color_eyre::eyre::Result;
use eyre::eyre;
use grid::grid::Timeline;

use protos::{location_proof::Proof, location_storage::{ObtainLocationReportRequest, Report, SubmitLocationReportRequest}};
use protos::location_storage::location_storage_client::LocationStorageClient;

async fn submit_location_report(idx : usize, epoch : usize, url : String, proofs_joined: &Vec<Proof>) -> Result<()> {

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(SubmitLocationReportRequest {
        idx: idx as u32,
        epoch: epoch as u32,
        report: Some(Report {
            proofs: proofs_joined.to_vec(),
        }),
    });

    match client.submit_location_report(request).await {
        Ok(_) => {
            Ok(())
        }
        Err(_) => { Err(eyre!("Something failed.")) }
    }
}


async fn obtain_location_report(idx : usize, epoch : usize, url : String) -> Result<(usize, usize)> {

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(ObtainLocationReportRequest {
        idx: idx as u32,
        epoch: epoch as u32,
    });

    match client.obtain_location_report(request).await {
        Ok(response) => {
            match Timeline::parse_valid_pos(response.get_ref().pos_x, response.get_ref().pos_y){
                Ok(pos) => { Ok(pos)},
                Err(err) => return Err(eyre!("Location not found."))
            }
        }
        Err(Status) => { Err(eyre!("Something failed.")) }
    }
}