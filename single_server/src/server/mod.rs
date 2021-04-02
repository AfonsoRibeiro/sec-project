pub mod validating;
pub mod management;

use color_eyre::eyre::Result;
use tonic::transport::Server;
use protos::location_storage::location_storage_server::LocationStorageServer;
use protos::location_master::location_master_server::LocationMasterServer;

pub async fn start_server(addr : String) -> Result<()> {
    let addr = addr.parse()?;
    let validater = validating::MyLocationStorage::default();
    let manager = management::MyLocationMaster::default();

    println!("LocationStorageServer listening on {}", addr);

    Server::builder()
        .add_service(LocationStorageServer::new(validater))
        .add_service(LocationMasterServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}