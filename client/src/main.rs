mod proofing_system;
mod reports;

use eyre::eyre;
use color_eyre::eyre::Result;

use futures::stream::{FuturesUnordered, StreamExt};
use futures::select;

use std::{collections::HashSet, sync::Arc, usize};
use structopt::StructOpt;
use regex::Regex;


use tokio::time::{interval_at, Duration, Instant};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tonic::transport::Uri;

use grid::grid::{Timeline, retrieve_timeline};
use security::{key_management::{
    ClientKeys,
    ServerPublicKey,
    retrieve_client_keys,
    retrieve_servers_public_keys,
}, proof::Proof, report::Report};

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {

    #[structopt(name = "id", long)]
    idx : usize,

    #[structopt(name = "grid", long, default_value = "grid/grid.txt")]
    grid_file : String,

    #[structopt(name = "keys", long, default_value = "security/keys")]
    keys_dir : String,

    #[structopt(name = "n_servers", long, default_value = "1")]
    n_servers : usize
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    let f_servers = (opt.n_servers - 1) / 3;
    let necessary_res= f_servers + opt.n_servers / 2;

    let timeline = Arc::new(retrieve_timeline(&opt.grid_file)?);

    if !timeline.is_point(opt.idx) {
        return Err(eyre!("Error : Invalid id for client {:}.", opt.idx));
    }

    let client_keys = Arc::new(retrieve_client_keys(&opt.keys_dir, opt.idx)?);
    let server_keys = Arc::new(retrieve_servers_public_keys(&opt.keys_dir)?);

    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");

    let proofer =
        tokio::spawn(proofing_system::start_proofer(opt.idx, timeline.clone(), client_keys.sign_key().clone()));

    let server_urls  = get_servers_url(opt.n_servers);

    tokio::spawn(epochs_generator(timeline.clone(), opt.idx, server_urls.clone(), client_keys.clone(), server_keys.clone(), necessary_res));

    read_commands(timeline.clone(), opt.idx, server_urls, client_keys, server_keys, necessary_res).await;

    let _x = proofer.await; // Not important result just dont end

    Ok(())
}

async fn reports_generator(
    timeline : Arc<Timeline>,
    idx : usize,
    epoch : usize,
    server_urls: Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_key : Arc<ServerPublicKey>,
    necessary_res : usize,
) {
    //TODO: server order -> random

    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(idx, epoch) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline.clone(), idx, epoch).await;
        if proofs.len() > timeline.f_line && proofs.len() == idxs_ass.len() {
            let report = Report::new(epoch, (loc_x, loc_y), idx, idxs_ass, proofs);

            let mut responses : FuturesUnordered<_> = server_urls.iter().enumerate().map(
                |(server_id, url)| reports::submit_location_report(
                    idx,
                    &report,
                    url,
                    client_keys.sign_key(),
                    server_key.public_key(server_id),
                )
            ).collect();

            let mut counter : usize = 0;
            loop {
                select! {
                    res = responses.select_next_some() => {
                        if res.is_ok() {
                            counter += 1;
                        }

                        if counter > necessary_res {
                            break ;
                        }
                    }
                    complete => break,
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
    necessary_res : usize,
) -> Result<()> {

    let start = Instant::now() + Duration::from_millis(2000);
    let mut interval = interval_at(start, Duration::from_millis(15000));

    for epoch in 0..timeline.epochs() {
        interval.tick().await;

        println!("Client {:} entered epoch {:}/{:}.", idx, epoch, timeline.epochs()-1);

        tokio::spawn(reports_generator(timeline.clone(), idx, epoch, server_urls.clone(), client_keys.clone(), server_keys.clone(), necessary_res));
    }
    Ok(())
}

async fn do_get_report_command(
    timeline : Arc<Timeline>,
    idx : usize,
    server_urls :  Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_keys : Arc<ServerPublicKey>,
    necessary_res : usize,
    epoch : usize,

) {
    let mut responses : FuturesUnordered<_> = server_urls.iter().enumerate().map(
        |(server_id, url)|
            reports::obtain_location_report(
                timeline.clone(),
                idx,
                epoch,
                url.clone(),
                client_keys.sign_key(),
                server_keys.public_key(server_id),
                client_keys.public_key()
            )
        ).collect();

    let mut locations : Vec<(usize, usize)> = Vec::with_capacity(necessary_res + 1);
    loop {
        select! {
            res = responses.select_next_some() => {
                if let Ok(loc) = res {
                    locations.push(loc);
                }

                if locations.len() > necessary_res {
                    break ;
                }
            }
            complete => break,
        }
    }

    println!("location {:?}", locations); // TODO only print most recent one

}

async fn do_get_proofs_command(
    idx : usize,
    server_urls :  Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_keys : Arc<ServerPublicKey>,
    necessary_res : usize,
    epochs : HashSet<usize>,
) {
    let mut responses : FuturesUnordered<_> = server_urls.iter().enumerate().map(
        |(server_id, url)|
            reports::request_my_proofs(
                idx,
                epochs.clone(),
                url.clone(),
                client_keys.sign_key(),
                server_keys.public_key(server_id),
                client_keys.public_key()
            )
        ).collect();

    let mut proofs_res : Vec<Vec<Proof>> = Vec::with_capacity(necessary_res + 1);
    loop {
        select! {
            res = responses.select_next_some() => {
                if let Ok(loc) = res {
                    proofs_res.push(loc);
                }

                if proofs_res.len() > necessary_res {
                    break ;
                }
            }
            complete => break,
        }
    }

    println!("{:?}", proofs_res);  // TODO only print most recent one
}

async fn read_commands(
    timeline : Arc<Timeline>,
    idx : usize,
    server_urls :  Arc<Vec<Uri>>,
    client_keys : Arc<ClientKeys>,
    server_keys : Arc<ServerPublicKey>,
    necessary_res : usize,
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

                do_get_report_command(
                    timeline.clone(),
                    idx,
                    server_urls.clone(),
                    client_keys.clone(),
                    server_keys.clone(),
                    necessary_res,
                    epoch.unwrap(),
                ).await

            } if rproofs_pat.is_match(buffer.trim_end()) {
                let mut epochs = HashSet::new();
                for epoch in buffer.trim_end().split(' ') {
                    if let Ok(epoch) = epoch.parse::<usize>() {
                        epochs.insert(epoch);
                    }
                }

                do_get_proofs_command(idx,
                    server_urls.clone(),
                    client_keys.clone(),
                    server_keys.clone(),
                    necessary_res,
                    epochs,
                ).await

            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() {
    println!("To obtain a report use: report <epoch>");
    println!("To obtain proofs recieved by server use: proof <epoch>+");
}

fn get_servers_url(n_servers : usize ) -> Arc<Vec<Uri>> {
    let mut server_urls = vec![];
    for i in 0..n_servers{
        server_urls.push(format!("http://[::1]:500{:02}", i).parse().unwrap());
    }
    Arc::new(server_urls)
}