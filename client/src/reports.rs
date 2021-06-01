use eyre::eyre;
use color_eyre::eyre::Result;

use std::{collections::HashSet};
use tonic::transport::Uri;

use protos::{location_storage::{ObtainLocationReportRequest, SubmitLocationReportRequest, RequestMyProofsRequest}};
use protos::location_storage::location_storage_client::LocationStorageClient;

use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use security::{proof::Proof, report::verify_report, status::{LocationReportRequest, MyProofsRequest, decode_my_proofs_response, decode_response_location, encode_location_report, encode_my_proofs_request}};
use security::report::{self, Report, success_report};

pub async fn submit_location_report(
    idx : usize,
    report : &Report,
    url : &Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
) -> Result<()> {

    let (report_info, report, key, pow) = report::encode_report(sign_key, server_key, report, idx);

    let mut client = LocationStorageClient::connect(url.clone()).await?;

    let request = tonic::Request::new(SubmitLocationReportRequest {
        report,
        report_info,
        pow,
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
        Err(status) => {
            println!("SubmitLocationReport failed with code {:?} and message {:?}.",
            status.code(), status.message());
            Err(eyre!("SubmitLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message()))
        }
    }
}


pub async fn obtain_location_report(
    idx : usize,
    epoch : usize,
    url : Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
    public_key : &sign::PublicKey,
)-> Result<(usize, usize)> {

    let loc_report = LocationReportRequest::new(idx, epoch);
    let (user_info, user, key, pow) = encode_location_report(sign_key, server_key, &loc_report, idx);

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(ObtainLocationReportRequest {
        user,
        user_info,
        pow,
    });

    let report = match client.obtain_location_report(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(res) = decode_response_location(&key, &response.nonce, &response.location) {
                if let Ok(report) = verify_report(public_key, &res.report) {
                    report
                } else {
                    return  Err(eyre!("obtain_location_report unable to verify report"));
                }
            } else {
                return Err(eyre!("obtain_location_report unable to validate server response "));
            }
        }
        Err(status) => return Err(eyre!("ObtainLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    if epoch == report.epoch(){
        Ok(report.loc())
    } else {
        Err(eyre!("Not the requested epoch: {:}", report.epoch()))
    }
}

pub async fn request_my_proofs(
    idx : usize,
    epochs : HashSet<usize>,
    url : Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
    public_key : &sign::PublicKey,
) -> Result<HashSet<Proof>> {

    let proofs_req = MyProofsRequest::new(epochs.clone());
    let (user_info, vec_epochs, key,pow) = encode_my_proofs_request(sign_key, server_key, &proofs_req, idx);

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(RequestMyProofsRequest {
        epochs : vec_epochs,
        user_info,
        pow,
    });

    let proofs = match client.request_my_proofs(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(proofs) = decode_my_proofs_response(&key, &public_key, &response.nonce, &response.proofs) {
                proofs
            } else {
                return Err(eyre!("obtain_location_report unable to validate server response "));
            }
        }
        Err(status) => return Err(eyre!("ObtainLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    for proof in proofs.iter() {
        if !epochs.contains(&proof.epoch()){
            return Err(eyre!("obtain_location_report unable to validate server response"));
        }
    }
    Ok(proofs.into_iter().collect())
}
