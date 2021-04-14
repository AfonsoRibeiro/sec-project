mod verifying;

use structopt::StructOpt;
use color_eyre::eyre::Result;
use regex::Regex;

use tonic::transport::Uri;
use tokio::io::{self, AsyncBufReadExt, BufReader};

use security::key_management::{HAClientKeys, ServerPublicKey, retrieve_ha_client_keys, retrieve_server_public_keys};

#[derive(StructOpt)]
#[structopt(name = "HA_Client", about = "Checking on server satus")]
struct Opt {

    #[structopt(name = "server", long, default_value = "http://[::1]:50051")]
    server_url : Uri,

    #[structopt(name = "size", long, default_value = "5")]
    grid_size : usize,

    #[structopt(name = "keys", long, default_value = "security/keys")]
    keys_dir : String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Starting HA_Client");

    let opt = Opt::from_args();

    let ha_keys = retrieve_ha_client_keys(&opt.keys_dir)?;
    let server_keys = retrieve_server_public_keys(&opt.keys_dir)?;

    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");

    read_commands(opt.grid_size, opt.server_url, &ha_keys, &server_keys).await;

    Ok(())
}

async fn read_commands(grid_size : usize, server : Uri, ha_keys : &HAClientKeys, server_keys : &ServerPublicKey) {
    print_command_msg();

    let o_rep_pat = Regex::new(r"r(eport)? [+]?(\d+) [+]?(\d+)").unwrap();
    let o_users_pat = Regex::new(r"u(sers)? [+]?(\d+) [+]?(\d+) [+]?(\d+)").unwrap();

    let mut reader = BufReader::new(io::stdin());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        reader.read_line(&mut buffer).await.unwrap(); // Trusting io (don know if it works with > )
        {
            if let Some(cap) = o_rep_pat.captures(buffer.trim_end()) {
                let id  = cap[2].parse::<usize>();
                let epoch  = cap[3].parse::<usize>();
                if id.is_err() || epoch.is_err() { print_command_msg(); continue; }

                match verifying::obtain_location_report(
                    id.unwrap(),
                    epoch.unwrap(),
                    grid_size,
                    server.clone(),
                    &ha_keys.sign_key(),
                    &server_keys.public_key()).await
                {
                    Ok((x, y)) => println!("location {:} {:}", x, y),
                    Err(err) => println!("{:}", err.to_string()),
                }

            } else if let Some(cap) = o_users_pat.captures(buffer.trim_end()) {
                let epoch  = cap[2].parse::<usize>();
                let pos_x  = cap[3].parse::<usize>();
                let pos_y  = cap[4].parse::<usize>();
                if epoch.is_err() || pos_x.is_err() || pos_y.is_err() { print_command_msg(); continue; }
                match verifying::obtain_users_at_location(epoch.unwrap(), pos_x.unwrap(), pos_y.unwrap(), server.clone()).await {
                    Ok(clients) => println!("clients {:?}", clients),
                    Err(err) => println!("{:}", err.to_string()),
                }
            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() { println!("To obtain a report use: report <id> <epoch>\nTo obtain users ate location use: users <epoch> <pos_x> <pos_y>"); }