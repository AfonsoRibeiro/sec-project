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

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {

    #[structopt(name = "server", long, default_value = "http://[::1]:50051")]
    server_url : Uri,

    #[structopt(name = "id", long)]
    idx : usize,

    #[structopt(name = "grid", long, default_value = "grid/grid.txt")]
    grid_file : String
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    let timeline = Arc::new(retrieve_timeline(&opt.grid_file)?);

    if !timeline.is_point(opt.idx) {
        return Err(eyre!("Error : Invalid id for client {:}.", opt.idx));
    }

    tokio::spawn(proofing_system::start_proofer(opt.idx, timeline.clone()));

    tokio::spawn(epochs_generator(timeline.clone(), opt.idx, opt.server_url.clone()));

    read_commands(timeline.clone(), opt.idx, opt.server_url).await;

    Ok(())
}

async fn reports_generator(timeline : Arc<Timeline>, idx : usize, epoch : usize, server_url : Uri) {
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(idx, epoch) {
        let proofs = proofing_system::get_proofs(timeline, idx, epoch).await;

        if proofs.len() > 0 {

                let _r = reports::submit_location_report(idx, epoch, loc_x, loc_y, server_url, proofs).await;  // If failed should we try and resubmit
            } else {
                println!("Client {:} unable to generate report for epoch {:}.", idx, epoch);
            }
    } else {
        print!("Error: reports_generator! (Should never happen)");
    }
}

async fn epochs_generator(timeline : Arc<Timeline>, idx : usize, server_url : Uri) -> Result<()> { //TODO: f', create report
    let start = Instant::now() + Duration::from_millis(50);
    let mut interval = interval_at(start, Duration::from_millis(5000));

    for epoch in 0..timeline.epochs() {
        interval.tick().await;

        println!("Client {:} entered epoch {:}/{:}.", idx, epoch, timeline.epochs()-1);

        tokio::spawn(reports_generator(timeline.clone(), idx, epoch, server_url.clone()));
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
        reader.read_line(&mut buffer).await.unwrap(); // Trusting io (don know if it works with > )
        {
            if let Some(cap) = orep_pat.captures(buffer.trim_end()) {
                let epoch  = cap[2].parse::<usize>();
                if epoch.is_err() { print_command_msg(); continue; }
                let _x = reports::obtain_location_report(timeline.clone(), idx, epoch.unwrap(), server.clone()).await;
                // TODO deal with return
            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() { println!("To obtain a report use: report <epoch>"); }