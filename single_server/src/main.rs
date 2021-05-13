mod server;
mod storage;

use color_eyre::eyre::Result;
use structopt::StructOpt;
use tonic::transport::Uri;

use std::{fs, sync::Arc};

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

    #[structopt(name = "n_servers", long, default_value = "1")]
    n_servers : usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Server starting");

    let opt = Opt::from_args();

    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");

    fs::create_dir_all(&opt.storage_dir)?;

    let storage_file = format!("{:}{:}.txt", &opt.storage_dir, opt.server_id);

    let storage = if let Ok(storage) = storage::retrieve_storage(&storage_file) {
        Arc::new(storage)
    } else{
        Arc::new( storage::Timeline::new(opt.grid_size, storage_file))
    };

    let server_keys = Arc::new(retrieve_server_keys(&opt.keys_dir, opt.server_id)?);

    let f_servers = (opt.n_servers - 1) / 3;
    let necessary_res= f_servers + opt.n_servers / 2;

    server::start_server(format!("[::1]:500{:02}", opt.server_id), 
        storage, 
        server_keys, 
        opt.f_line,
        get_servers_url(opt.n_servers, opt.server_id),
        necessary_res,
        f_servers
    ).await?;

    Ok(())
}

fn get_servers_url(n_servers : usize , id : usize) -> Arc<Vec<Uri>> {
    let mut server_urls = vec![];
    for i in 0..n_servers{
        if i != id {
            server_urls.push(format!("http://[::1]:500{:02}", i).parse().unwrap());    
        }
    }
    Arc::new(server_urls)
}
