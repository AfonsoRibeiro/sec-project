use std::collections::{HashMap, HashSet};

use eyre::eyre;
use color_eyre::eyre::Result;

use sodiumoxide::crypto::{box_, sign};
use status::{UsersAtLocationRequest, encode_location_report, encode_users_at_location_report};
use tonic::transport::Uri;

use security::{report, status::{self, LocationReportRequest}};

use protos::location_master::location_master_client::LocationMasterClient;
use protos::location_master::{ObtainLocationReportRequest, ObtainUsersAtLocationRequest};


pub async fn obtain_location_report(
    idx : usize,
    epoch : usize,
    grid_size : usize,
    url : Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
    client_public_key : &sign::PublicKey
) -> Result<(usize, usize)> {

    let mut client = LocationMasterClient::connect(url).await?;

    let loc_report = LocationReportRequest::new(idx, epoch);
    let (info, user, key, pow) = encode_location_report(&sign_key, server_key, &loc_report, idx);

    let request = tonic::Request::new(ObtainLocationReportRequest {
        user,
        info,
        pow,
    });

    let report = match client.obtain_location_report(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(res) = status::decode_response_location(&key, &response.nonce, &response.location) {
                if let Ok(report) = report::verify_report(client_public_key, &res.report) {
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

    let (x, y) = report.loc();
    if x < grid_size && y < grid_size && epoch == report.epoch(){
        Ok((x, y))
    } else {
        Err(eyre!("Response : Not a valid position (x : {:}, y : {:}) or not the same epoch!", x, y))
    }
}

pub async fn obtain_users_at_location(
    epoch : usize,
    pos_x : usize,
    pos_y : usize,
    url : Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
    clients_public_keys : &HashMap<usize, sign::PublicKey>
) -> Result<HashSet<usize>> {

    let mut client = LocationMasterClient::connect(url).await?;

    let loc_report = UsersAtLocationRequest::new((pos_x, pos_y), epoch);
    let (info, place, key, pow) = encode_users_at_location_report(&sign_key, server_key, &loc_report, 0);

    let request = tonic::Request::new(ObtainUsersAtLocationRequest {
        place,
        info,
        pow,
    });

    match client.obtain_users_at_location(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(res) = status::decode_users_at_loc_response(&key, &response.nonce, &response.idxs) {
                let mut idxs : HashSet<usize> = HashSet::new();
                for (idx, report) in res.idxs_reports.iter() {
                    if !clients_public_keys.contains_key(idx) {
                        return Err(eyre!("obtain_location_report unable to find user"));
                    }
                    if let Ok(report) = report::verify_report(clients_public_keys.get(idx).unwrap(), report) {
                        if report.epoch() == epoch && report.loc() == (pos_x, pos_y) {
                            idxs.insert(*idx);
                        } else {
                            return Err(eyre!("obtain_location_report unable to validate all users reports"));
                        }
                    }  else {
                        return Err(eyre!("obtain_location_report unable to validate all users reports"));
                    }
                }
                Ok(idxs)
            } else {
                return Err(eyre!("obtain_location_report unable to validate server response "));
            }
        }
        Err(status) => return Err(eyre!("ObtainUsersAtLocation failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    }
}