mod proofing_system;
mod reports;

use eyre::eyre;
use color_eyre::eyre::Result;

use std::sync::Arc;
use structopt::StructOpt;
use regex::Regex;


use tokio::time::{interval_at, Duration, Instant};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tonic::transport::Uri;

use grid::grid::{Timeline, retrieve_timeline};
use security::key_management::{
    ClientKeys,
    ServerPublicKey,
    retrieve_client_keys,
    retrieve_server_public_keys,
};

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {

    #[structopt(name = "server", long, default_value = "http://[::1]:50051")]
    server_url : Uri,

    #[structopt(name = "id", long)]
    idx : usize,

    #[structopt(name = "grid", long, default_value = "grid/grid.txt")]
    grid_file : String,

    #[structopt(name = "keys", long, default_value = "security/keys")]
    keys_dir : String,

    #[structopt(name = "fline", long, default_value = "3")]
    f_line : usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    let timeline = Arc::new(retrieve_timeline(&opt.grid_file)?);

    let client_keys = retrieve_client_keys(&opt.keys_dir, opt.idx)?;
    let server_keys = retrieve_server_public_keys(&opt.keys_dir)?;

    if !timeline.is_point(opt.idx) {
        return Err(eyre!("Error : Invalid id for client {:}.", opt.idx));
    }

    if sodiumoxide::init().is_err() {
        return Err(eyre!("Unable to make sodiumoxide thread safe"));
    }

    let proofer =
        tokio::spawn(proofing_system::start_proofer(opt.idx, timeline.clone(), client_keys.sign_key()));

    tokio::spawn(epochs_generator(timeline.clone(), opt.idx, opt.server_url.clone(), client_keys, server_keys, opt.f_line));

    read_commands(timeline.clone(), opt.idx, opt.server_url).await;

    let _x = proofer.await; // Not important result just dont end

    Ok(())
}

async fn reports_generator(
    timeline : Arc<Timeline>,
    idx : usize,
    epoch : usize,
    server_url : Uri,
    client_keys : Arc<ClientKeys>,
    server_key : Arc<ServerPublicKey>,
    f_line : usize ) {

    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(idx, epoch) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline, idx, epoch, f_line).await;
        if proofs.len() > 0 && proofs.len() == idxs_ass.len() {
            let _r = reports::submit_location_report(
                idx,
                epoch,
                (loc_x, loc_y),
                server_url,
                proofs,
                idxs_ass,
                client_keys.sign_key(),
                client_keys.private_key(),
                server_key.public_key()
            ).await;  // TODO FIX If failed should we try and resubmit

            } else {
                println!("Client {:} unable to generate report for epoch {:}.", idx, epoch);
            }
    } else {
        print!("Error: reports_generator! (Should never happen)");
    }
}

async fn epochs_generator(
    timeline : Arc<Timeline>,
    idx : usize,
    server_url : Uri,
    client_keys : ClientKeys,
    server_keys : ServerPublicKey,
    f_line : usize,
) -> Result<()> {

    let client_keys = Arc::new(client_keys);
    let server_keys = Arc::new(server_keys);

    let start = Instant::now() + Duration::from_millis(2000);
    let mut interval = interval_at(start, Duration::from_millis(15000));

    for epoch in 0..timeline.epochs() {
        interval.tick().await;

        println!("Client {:} entered epoch {:}/{:}.", idx, epoch, timeline.epochs()-1);

        tokio::spawn(reports_generator(timeline.clone(), idx, epoch, server_url.clone(), client_keys.clone(), server_keys.clone(), f_line));
    }
    Ok(())
}


async fn read_commands(timeline : Arc<Timeline>, idx : usize, server : Uri) {
    print_command_msg();

    let orep_pat = Regex::new(r"r(eport)? [+]?(\d+)").unwrap();

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
                match reports::obtain_location_report(timeline.clone(), idx, epoch.unwrap(), server.clone()).await {
                    Ok((x, y)) => println!("location {:} {:}", x, y),
                    Err(err) => println!("{:}", err.to_string()),
                }

            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() { println!("To obtain a report use: report <epoch>"); }