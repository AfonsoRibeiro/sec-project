use color_eyre::eyre::Result;
use eyre::eyre;

use protos::location_storage::{SubmitLocationReportRequest, ObtainLocationReportRequest};
use protos::location_storage::location_storage_client::LocationStorageClient;


async fn submit_location_report(idx : usize, epoch : usize, url : String) -> Result<()> {

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(SubmitLocationReportRequest {
        idx: idx as u32,
        epoch: epoch as u32,
        report: None,
    });

    match client.submit_location_report(request).await {
        Ok(_) => {
            Ok(())
        }
        Err(_) => { Err(eyre!("Something failed.")) }
    }
}


async fn obtain_location_report(idx : usize, epoch : usize, url : String) -> Result<()> {

    let mut client = LocationStorageClient::connect(url).await?;

    let request = tonic::Request::new(ObtainLocationReportRequest {
        idx: idx as u32,
        epoch: epoch as u32,
    });

    let response = client.obtain_location_report(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())

}