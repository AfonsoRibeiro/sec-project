pub mod validating;
pub mod management;

use color_eyre::eyre::Result;
use tonic::transport::Server;
use protos::location_storage::location_storage_server::LocationStorageServer;

pub async fn start_server(addr : String) -> Result<()> {
    let addr = addr.parse()?;
    let validater = validating::MyLocationStorage::default();
    //let manager = management::MyLocationMaster::default();

    println!("LocationStorageServer listening on {}", addr);

    Server::builder()
        .add_service(LocationStorageServer::new(validater))
        //.add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}