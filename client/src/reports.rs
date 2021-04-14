use eyre::eyre;
use color_eyre::eyre::Result;
use status::encode_location_report;

use std::sync::Arc;
use std::convert::TryFrom;
use tonic::transport::Uri;

use grid::grid::Timeline;

use protos::{location_storage::{ObtainLocationReportRequest, SubmitLocationReportRequest}};
use protos::location_storage::location_storage_client::LocationStorageClient;

use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use security::status::{self, LocationReportRequest, LocationReportResponse};
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


pub async fn obtain_location_report(
    timeline : Arc<Timeline>,
    idx : usize,
    epoch : usize,
    url : Uri,
    sign_key : sign::SecretKey,
    server_key : box_::PublicKey,
)-> Result<(usize, usize)> {

    let loc_report = LocationReportRequest::new(idx, epoch);
    let (user_info, user, key) = encode_location_report(&sign_key, &server_key, &loc_report, idx);

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(ObtainLocationReportRequest {
        user,
        user_info
    });

    let loc = match client.obtain_location_report(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(x) = status::decode_response_location(&key, &response.nonce, &response.location) {
                x
            } else {
                return Err(eyre!("obtain_location_report unable to validate server response "));
            }
        }
        Err(status) => return Err(eyre!("ObtainLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    let (x, y) = loc.pos;
    if timeline.valid_pos(x, y) {
        Ok((x, y))
    } else {
        Err(eyre!("Response : Not a valid position (x : {:}, y : {:})", x, y))
    }
}