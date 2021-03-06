mod server;
mod storage;

use color_eyre::eyre::Result;
use structopt::StructOpt;
use tonic::transport::Uri;

use std::{fs, sync::Arc};

use security::key_management::{retrieve_server_keys, retrieve_servers_public_keys};

#[derive(StructOpt)]
#[structopt(name = "Server", about = "(Highly) Dependable Location Tracker")]
struct Opt {

    #[structopt(name = "size", long, default_value = "5")]
    grid_size : usize,

    #[structopt(name = "keys", long, default_value = "security/keys/")]
    keys_dir : String,

    #[structopt(name = "storage", long, default_value = "server/storage/")]
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
    let server_pkeys = Arc::new(retrieve_servers_public_keys(&opt.keys_dir)?);

    let f_servers = (opt.n_servers - 1) / 3;
    let necessary_res= f_servers + opt.n_servers / 2;

    server::start_server(
        opt.server_id,
        format!("[::1]:500{:02}", opt.server_id),
        storage,
        server_keys,
        opt.f_line,
        get_servers_url(opt.n_servers, opt.server_id),
        necessary_res,
        f_servers,
        server_pkeys,
    ).await?;

    Ok(())
}

fn get_servers_url(n_servers : usize, server_id : usize) -> Vec<(usize, Uri)> {
    let mut server_urls = vec![];
    for i in 0..n_servers{
        if i == server_id {continue;}
        server_urls.push((i, format!("http://[::1]:500{:02}", i).parse().unwrap()));
    }
    server_urls
}
