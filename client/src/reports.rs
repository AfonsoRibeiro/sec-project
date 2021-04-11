use eyre::eyre;
use color_eyre::eyre::Result;

use std::sync::Arc;
use std::convert::TryFrom;
use tonic::transport::Uri;

use grid::grid::Timeline;

use protos::{location_storage::{ObtainLocationReportRequest, SubmitLocationReportRequest}};
use protos::location_storage::location_storage_client::LocationStorageClient;

use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;

use security::report::{self, Report};


pub async fn submit_location_report(
    idx : usize,
    epoch : usize,
    loc : (usize, usize),
    url : Uri,
    proofs_joined: Vec<Vec<u8>>,
    idxs_ass : Vec<usize>,
    sign_key : sign::SecretKey,
    private_key : box_::SecretKey,
    server_key : box_::PublicKey,
) -> Result<()> {

    let report = Report::new(epoch, loc, idx, idxs_ass, proofs_joined);

    let (report, nonce) = report::encode_report(&sign_key, &private_key, &server_key, report);

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(SubmitLocationReportRequest {
        idx : idx as u64,
        nonce : nonce.0.to_vec(),
        report,
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
        idx: idx as u64,
        epoch: epoch as u64,
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