mod server;
mod storage;

use structopt::StructOpt;
use color_eyre::eyre::Result;
use std::sync::Arc;

use security::key_management::retrieve_server_keys;

#[derive(StructOpt)]
#[structopt(name = "Single Server", about = "(Highly) Dependable Location Tracker")]
struct Opt {

    #[structopt(name = "server", long, default_value = "[::1]:50051")]
    server_addr : String,

    #[structopt(name = "size", long, default_value = "5")]
    grid_size : usize,

    #[structopt(name = "keys", long, default_value = "security/keys/")]
    keys_dir : String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Server starting");

    let opt = Opt::from_args();

    let storage = Arc::new( storage::Timeline::new(opt.grid_size) );
    let server_keys = Arc::new(retrieve_server_keys(&opt.keys_dir)?);

    server::start_server(opt.server_addr, storage, server_keys).await?;

    Ok(())
}
