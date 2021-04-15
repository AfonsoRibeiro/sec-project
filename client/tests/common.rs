use std::sync::Arc;

use grid::grid::{Timeline, retrieve_timeline};
use security::key_management::{
    ClientKeys,
    retrieve_client_keys,
    retrieve_server_keys,
};

use sodiumoxide::crypto::sign;


const KEYS_DIR : &str = "../security/keys";
const GRID_FILE : &str = "../grid/grid.txt";

pub fn get_timeline() -> Arc<Timeline> {
    Arc::new(retrieve_timeline(GRID_FILE).expect("Failed to retrieve timeline"))
}

pub fn get_client_keys(idx : usize) -> Arc<ClientKeys> {
    Arc::new(retrieve_client_keys(KEYS_DIR, idx).expect("Failed to retrieve sign key"))
}

pub fn get_pub_sign_key(idx : usize) -> sign::PublicKey  {
    let server_keys = retrieve_server_keys(KEYS_DIR).expect("Unhable to get server keys");
    *server_keys.client_sign_key(idx).unwrap()
}

pub fn setup_reporting() {
    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");
}