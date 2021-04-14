use eyre::eyre;
use color_eyre::eyre::Result;

use sodiumoxide::crypto::{box_, sign};
use status::{UsersAtLocationRequest, encode_location_report, encode_users_at_location_report};
use tonic::transport::Uri;

use security::status::{self, LocationReportRequest};

use protos::location_master::location_master_client::LocationMasterClient;
use protos::location_master::{ObtainLocationReportRequest, ObtainUsersAtLocationRequest};


pub async fn obtain_location_report(
    idx : usize,
    epoch : usize,
    grid_size : usize,
    url : Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
) -> Result<(usize, usize)> {

    let mut client = LocationMasterClient::connect(url).await?;

    let loc_report = LocationReportRequest::new(idx, epoch);
    let (info, user, key) = encode_location_report(&sign_key, server_key, &loc_report, idx);

    let request = tonic::Request::new(ObtainLocationReportRequest {
        user,
        info
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
    if x < grid_size && y < grid_size {
        Ok((x, y))
    } else {
        Err(eyre!("Response : Not a valid position (x : {:}, y : {:})", x, y))
    }
}

pub async fn obtain_users_at_location(
    epoch : usize,
    pos_x : usize,
    pos_y : usize,
    url : Uri,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
) -> Result<Vec<usize>> {

    let mut client = LocationMasterClient::connect(url).await?;

    let loc_report = UsersAtLocationRequest::new((pos_x, pos_y), epoch);
    let (info, place, key) = encode_users_at_location_report(&sign_key, server_key, &loc_report, 0);

    let request = tonic::Request::new(ObtainUsersAtLocationRequest {
        place,
        info
    });

    match client.obtain_users_at_location(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if let Ok(idxs) = status::decode_users_at_loc_response(&key, &response.nonce, &response.idxs) {
                Ok(idxs.idxs)
            } else {
                return Err(eyre!("obtain_location_report unable to validate server response "));
            }
        }
        Err(status) => return Err(eyre!("ObtainUsersAtLocation failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    }
}