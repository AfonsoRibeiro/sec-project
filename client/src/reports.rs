use eyre::eyre;
use color_eyre::eyre::Result;

use std::sync::Arc;
use std::convert::TryFrom;
use tonic::transport::Uri;

use grid::grid::Timeline;

use protos::{location_proof::Proof, location_storage::{ObtainLocationReportRequest, Report, SubmitLocationReportRequest}};
use protos::location_storage::location_storage_client::LocationStorageClient;

pub async fn submit_location_report(
    idx : usize,
    epoch : usize,
    loc_x : usize,
    loc_y : usize,
    url : Uri,
    proofs_joined: Vec<Proof>
) -> Result<()> {

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(SubmitLocationReportRequest {
        idx : idx as u32,
        epoch : epoch as u32,
        loc_x : loc_x as u32,
        loc_y : loc_y as u32,
        report : Some(Report {
            proofs: proofs_joined,
        }),
    });

    match client.submit_location_report(request).await {
        Ok(_) => Ok(()),
        Err(status) => Err(eyre!("SubmitLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    }
}


pub async fn obtain_location_report(timeline : Arc<Timeline>, idx : usize, epoch : usize, url : Uri) -> Result<(usize, usize)> {

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(ObtainLocationReportRequest {
        idx: idx as u32,
        epoch: epoch as u32,
    });

    let response = match client.obtain_location_report(request).await {
        Ok(response) => response,
        Err(status) => return Err(eyre!("ObtainLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    let x: Result<usize, std::num::TryFromIntError> = usize::try_from(response.get_ref().pos_x);
    let y = usize::try_from(response.get_ref().pos_y);

    if x.is_err() || y.is_err() {
        return Err(eyre!("Response : Not a valid x ou y value"));
    }

    let (x, y) = (x.unwrap(), y.unwrap());
    if timeline.valid_pos(x, y) {
        Ok((x, y))
    } else {
        Err(eyre!("Response : Not a valid position (x : {:}, y : {:})", x, y))
    }
}