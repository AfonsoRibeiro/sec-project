use eyre::eyre;
use color_eyre::eyre::Result;

use tonic::transport::Uri;


use protos::location_master::location_master_client::LocationMasterClient;
use protos::location_master::{ObtainLocationReportRequest, ObtainLocationReportResponse,
    ObtainUsersAtLocationRequest, ObtainUsersAtLocationResponse};


pub async fn obtain_location_report(idx : u64, epoch : u64, grid_size : u64, url : Uri) -> Result<(u64, u64)> {

    let mut client = LocationMasterClient::connect(url).await?;

    let request = tonic::Request::new(ObtainLocationReportRequest {
        idx: idx as u64,
        epoch: epoch as u64,
    });

    let response = match client.obtain_location_report(request).await {
        Ok(response) => response,
        Err(status) => return Err(eyre!("ObtainLocationReport failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    let (x, y) = (response.get_ref().pos_x, response.get_ref().pos_y);
    if x < grid_size && y < grid_size {
        Ok((x, y))
    } else {
        Err(eyre!("Response : Not a valid position (x : {:}, y : {:})", x, y))
    }
}

pub async fn obtain_users_at_location(epoch : u64, pos_x : u64, pos_y : u64, url : Uri) -> Result<Vec<u64>> {

    let mut client = LocationMasterClient::connect(url).await?;

    let request = tonic::Request::new(ObtainUsersAtLocationRequest {
        epoch,
        pos_x,
        pos_y
    });

    let response = match client.obtain_users_at_location(request).await {
        Ok(response) => response,
        Err(status) => return Err(eyre!("ObtainUsersAtLocation failed with code {:?} and message {:?}.",
                            status.code(), status.message())),
    };

    Ok(response.get_ref().idxs.clone())
}