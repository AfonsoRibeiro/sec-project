use std::sync::Arc;

use grid::grid::{Timeline, retrieve_timeline};
use security::key_management::{
    ClientKeys,
    retrieve_client_keys,
    retrieve_server_keys,
    retrieve_servers_public_keys,
};

use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;


const KEYS_DIR : &str = "../security/keys";
const GRID_FILE : &str = "../grid/grid.txt";

#[allow(dead_code)]
pub fn make_thread_safe() {
    sodiumoxide::init().expect("Unhable to make it thread safe");
}

pub fn get_timeline() -> Arc<Timeline> {
    Arc::new(retrieve_timeline(GRID_FILE).expect("Failed to retrieve timeline"))
}

#[allow(dead_code)]
pub fn get_client_keys(idx : usize) -> Arc<ClientKeys> {
    Arc::new(retrieve_client_keys(KEYS_DIR, idx).expect("Failed to retrieve sign key"))
}

#[allow(dead_code)]
pub fn get_pub_sign_key(idx : usize) -> sign::PublicKey  {
    let server_keys = retrieve_server_keys(KEYS_DIR, 0).expect("Unhable to get server keys");
    *server_keys.client_sign_key(idx).unwrap()
}

#[allow(dead_code)]
pub fn get_pub_server_key() ->  Vec<box_::PublicKey> {
    retrieve_servers_public_keys(KEYS_DIR).unwrap().public_keys().to_vec()
}
