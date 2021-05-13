pub mod validating;
pub mod management;
pub mod double_echo_report;

use std::sync::Arc;

use color_eyre::eyre::Result;
use tonic::transport::{Server, Uri};
use protos::{double_echo_broadcast::double_echo_broadcast_server::DoubleEchoBroadcastServer, location_storage::location_storage_server::LocationStorageServer};
use protos::location_master::location_master_server::LocationMasterServer;

use crate::storage::Timeline;
use security::key_management::ServerKeys;

pub async fn start_server(addr : String, storage : Arc<Timeline>, 
    server_keys : Arc<ServerKeys>,
    f_line : usize, 
    server_urls :  Arc<Vec<Uri>>, 
    necessary_res : usize,
    f_servers : usize
) -> Result<()> {

    let addr = addr.parse()?;
    let validater = validating::MyLocationStorage::new(storage.clone(), server_keys.clone(), f_line, server_urls.clone(), necessary_res, f_servers);
    let manager = management::MyLocationMaster::new(storage.clone(), server_keys);
    let echo = double_echo_report::MyDoubleEchoReport::new(server_urls, necessary_res, f_servers);
    println!("LocationStorageServer listening on {}", addr);

    Server::builder()
        .add_service(LocationStorageServer::new(validater))
        .add_service(LocationMasterServer::new(manager))
        .add_service(DoubleEchoBroadcastServer::new(echo))
        .serve(addr)
        .await?;

    Ok(())
}

