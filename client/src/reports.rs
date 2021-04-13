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

use security::report::{self, Report, success_report};


pub async fn submit_location_report(
    idx : usize,
    report : &Report,
    url : Uri,
    sign_key : sign::SecretKey,
    server_key : box_::PublicKey,
) -> Result<()> {

    let (report_info, report, key) = report::encode_report(&sign_key, &server_key, report, idx);

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(SubmitLocationReportRequest {
        report,
        report_info,
    });

    match client.submit_location_report(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if success_report(&key, &response.nonce, &response.ok) {
                Ok(())
            } else {
                Err(eyre!("submit_location_report unable to validate server response "))
            }
        }
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