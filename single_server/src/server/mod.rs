pub mod validating;
pub mod management;

use std::sync::Arc;

use color_eyre::eyre::Result;
use tonic::transport::Server;
use protos::location_storage::location_storage_server::LocationStorageServer;
use protos::location_master::location_master_server::LocationMasterServer;

use crate::storage::Timeline;
use security::key_management::ServerKeys;

pub async fn start_server(addr : String, storage : Arc<Timeline>, server_keys : Arc<ServerKeys>, f_line : usize) -> Result<()> {
    let addr = addr.parse()?;
    let validater = validating::MyLocationStorage::new(storage.clone(), server_keys.clone(), f_line);
    let manager = management::MyLocationMaster::new(storage.clone(), server_keys);

    println!("LocationStorageServer listening on {}", addr);

    Server::builder()
        .add_service(LocationStorageServer::new(validater))
        .add_service(LocationMasterServer::new(manager))
        .serve(addr)
        .await?;

    Ok(())
}