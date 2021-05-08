mod proofing_system;
mod reports;

use eyre::eyre;
use color_eyre::eyre::Result;

use std::{collections::HashSet, sync::Arc, usize};
use structopt::StructOpt;
use regex::Regex;


use tokio::time::{interval_at, Duration, Instant, sleep};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tonic::transport::Uri;

use grid::grid::{Timeline, retrieve_timeline};
use security::{key_management::{
    ClientKeys,
    ServerPublicKey,
    retrieve_client_keys,
    retrieve_servers_public_keys,
}, report::Report};

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {

    #[structopt(name = "id", long)]
    idx : usize,

    #[structopt(name = "grid", long, default_value = "grid/grid.txt")]
    grid_file : String,

    #[structopt(name = "keys", long, default_value = "security/keys")]
    keys_dir : String,

    #[structopt(name = "n_servers", long, default_value = "0")]
    n_servers : usize
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    let timeline = Arc::new(retrieve_timeline(&opt.grid_file)?);

    if !timeline.is_point(opt.idx) {
        return Err(eyre!("Error : Invalid id for client {:}.", opt.idx));
    }

    let client_keys = Arc::new(retrieve_client_keys(&opt.keys_dir, opt.idx)?);
    let server_keys = Arc::new(retrieve_servers_public_keys(&opt.keys_dir)?);

    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");

    let proofer =
        tokio::spawn(proofing_system::start_proofer(opt.idx, timeline.clone(), client_keys.sign_key().clone()));

    let server_url  = get_servers_url(opt.n_servers);

    println!("{:?}", server_url);
    tokio::spawn(epochs_generator(timeline.clone(), opt.idx, server_url.clone(), client_keys.clone(), server_keys.clone()));

    read_commands(timeline.clone(), opt.idx, server_url, client_keys, server_keys).await;

    let _x = proofer.await; // Not important result just dont end

    Ok(())
}

async fn reports_generator(
    timeline : Arc<Timeline>,
    idx : usize,
    epoch : usize,
    server_urls: Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_key : Arc<ServerPublicKey>, ) {

    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(idx, epoch) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline.clone(), idx, epoch).await;
        if proofs.len() > timeline.f_line && proofs.len() == idxs_ass.len() {
            let report = Report::new(epoch, (loc_x, loc_y), idx, idxs_ass, proofs);
            for (server_id, server_url) in server_urls.iter().enumerate() {
                println!("connecting to {:?}", server_url);
                while reports::submit_location_report(
                    idx,
                    &report,
                    server_url,
                    client_keys.sign_key(),
                    server_key.public_key(server_id),
                ).await.is_err() {
                    println!("Unhable to submit report to server {:?}", server_url.clone());
                    sleep(Duration::from_millis(500)).await; // allow time for server recovery
                }
            }
        } else {
            println!("Client {:} unable to generate report for epoch {:}.", idx, epoch);
        }
    } else {
        println!("Error: reports_generator! (Should never happen)");
    }
}

async fn epochs_generator(
    timeline : Arc<Timeline>,
    idx : usize,
    server_urls : Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_keys : Arc<ServerPublicKey>,
) -> Result<()> {

    let start = Instant::now() + Duration::from_millis(2000);
    let mut interval = interval_at(start, Duration::from_millis(15000));

    for epoch in 0..timeline.epochs() {
        interval.tick().await;

        println!("Client {:} entered epoch {:}/{:}.", idx, epoch, timeline.epochs()-1);

        tokio::spawn(reports_generator(timeline.clone(), idx, epoch, server_urls.clone(), client_keys.clone(), server_keys.clone()));
    }
    Ok(())
}


async fn read_commands(
    timeline : Arc<Timeline>,
    idx : usize,
    server_urls :  Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_keys : Arc<ServerPublicKey>,
){
    print_command_msg();

    let orep_pat = Regex::new(r"r(eport)? [+]?(\d+)").unwrap();
    let rproofs_pat = Regex::new(r"p(roofs)?( [+]?(\d)+)+").unwrap(); // FIX TODO

    let mut reader = BufReader::new(io::stdin());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        if reader.read_line(&mut buffer).await.unwrap() == 0 {
            break;
        }
        {
            if let Some(cap) = orep_pat.captures(buffer.trim_end()) {
                let epoch  = cap[2].parse::<usize>();
                if epoch.is_err() { print_command_msg(); continue; }
                match reports::obtain_location_report(timeline.clone(), idx, epoch.unwrap(), server_urls[0].clone(), client_keys.sign_key(), server_keys.public_key(0)).await {
                    Ok((x, y)) => println!("location {:} {:}", x, y),
                    Err(err) => println!("{:}", err.to_string()),
                }

            } if rproofs_pat.is_match(buffer.trim_end()) {
                let mut epochs = HashSet::new();
                for epoch in buffer.split(' ') {
                    if let Ok(epoch) = epoch.parse::<usize>() {
                        epochs.insert(epoch);
                    }
                }
                match reports::request_my_proofs(idx, epochs, server_urls[0].clone(), client_keys.sign_key(), server_keys.public_key(0)).await {
                    Ok(()) => println!("proofs"),
                    Err(err) => println!("{:}", err.to_string()),
                }            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() { println!("To obtain a report use: report <epoch>"); }

fn get_servers_url(n_servers : usize ) -> Arc<Vec<Uri>> {
    let mut server_urls = vec![];
    for i in 0..n_servers{
        server_urls.push(format!("http://[::1]:500{:02}", i).parse().unwrap());
    }
    Arc::new(server_urls)
}