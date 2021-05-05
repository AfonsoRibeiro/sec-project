use eyre::eyre;
use color_eyre::eyre::Result;

use std::{collections::HashSet, sync::Arc};
use tonic::transport::Uri;

use grid::grid::Timeline;

use protos::{location_storage::{ObtainLocationReportRequest, SubmitLocationReportRequest, RequestMyProofsRequest}};
use protos::location_storage::location_storage_client::LocationStorageClient;

use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use security::status::{LocationReportRequest, MyProofsRequest, decode_my_proofs_response, decode_response_location, encode_location_report, encode_my_proofs_request};
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
            if let Ok(x) = decode_response_location(&key, &response.nonce, &response.location) {
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

pub async fn request_my_proofs(
    idx : usize,
    epochs : HashSet<usize>,
    url : Uri,
    sign_key : sign::SecretKey,
    server_key : box_::PublicKey,
) -> Result<()> {

    let proofs_req = MyProofsRequest::new(epochs);
    let (user_info, epochs, key) = encode_my_proofs_request(&sign_key, &server_key, &proofs_req, idx);

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(RequestMyProofsRequest {
        epochs,
        user_info,
    });

    let signed_proofs = match client.request_my_proofs(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(x) = decode_my_proofs_response(&key, &response.nonce, &response.proofs) {
                x
            } else {
                return Err(eyre!("obtain_location_report unable to validate server response "));
            }
        }
        Err(status) => return Err(eyre!("ObtainLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    // TODO do something with proofs and verify them

    Ok(())
}
