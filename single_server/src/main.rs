mod server;
mod storage;

use color_eyre::eyre::Result;
use structopt::StructOpt;

use std::sync::Arc;

use security::key_management::retrieve_server_keys;

#[derive(StructOpt)]
#[structopt(name = "Single Server", about = "(Highly) Dependable Location Tracker")]
struct Opt {

    #[structopt(name = "size", long, default_value = "5")]
    grid_size : usize,

    #[structopt(name = "keys", long, default_value = "security/keys/")]
    keys_dir : String,

    #[structopt(name = "storage", long, default_value = "single_server/storage/")]
    storage_dir : String,

    #[structopt(name = "id", long)]
    server_id: usize,

    #[structopt(name = "fline", long, default_value = "3")]
    f_line : usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Server starting");

    let opt = Opt::from_args();

    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");

    let storage_file = format!("{:}{:}.txt", &opt.storage_dir, opt.server_id);

    let storage = if let Ok(storage) = storage::retrieve_storage(&storage_file) {
        Arc::new(storage)
    } else{
        Arc::new( storage::Timeline::new(opt.grid_size, storage_file))
    };

    let server_keys = Arc::new(retrieve_server_keys(&opt.keys_dir)?);

    server::start_server(format!("[::1]:500{:02}", opt.server_id), storage, server_keys, opt.f_line).await?;

    Ok(())
}
