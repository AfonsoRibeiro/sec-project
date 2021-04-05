pub mod validating;
pub mod management;

use std::sync::Arc;

use color_eyre::eyre::Result;
use tonic::transport::Server;
use protos::location_storage::location_storage_server::LocationStorageServer;
use protos::location_master::location_master_server::LocationMasterServer;

use crate::storage::Timeline;

pub async fn start_server(addr : String, storage : Arc<Timeline>) -> Result<()> {
    let addr = addr.parse()?;
    let validater = validating::MyLocationStorage::new(storage.clone());
    let manager = management::MyLocationMaster::new(storage);

    println!("LocationStorageServer listening on {}", addr);

    Server::builder()
        .add_service(LocationStorageServer::new(validater))
        .add_service(LocationMasterServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}