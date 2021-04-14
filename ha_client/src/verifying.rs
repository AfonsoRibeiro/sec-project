use eyre::eyre;
use color_eyre::eyre::Result;

use sodiumoxide::crypto::{box_, sign};
use status::encode_location_report;
use tonic::transport::Uri;

use security::status::{self, LocationReportRequest, LocationReportResponse};

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
    let (user_info, user, key) = encode_location_report(&sign_key, server_key, &loc_report, idx);

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
    if x < grid_size && y < grid_size {
        Ok((x, y))
    } else {
        Err(eyre!("Response : Not a valid position (x : {:}, y : {:})", x, y))
    }
}

pub async fn obtain_users_at_location(epoch : usize, pos_x : usize, pos_y : usize, url : Uri) -> Result<Vec<u64>> {

    let mut client = LocationMasterClient::connect(url).await?;

    let request = tonic::Request::new(ObtainUsersAtLocationRequest {
        epoch : epoch as u64,
        pos_x : pos_x as u64,
        pos_y : pos_y as u64,
    });

    let response = match client.obtain_users_at_location(request).await {
        Ok(response) => response,
        Err(status) => return Err(eyre!("ObtainUsersAtLocation failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    Ok(response.get_ref().idxs.clone())
}