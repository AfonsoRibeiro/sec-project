mod proofing_system;
mod reports;

use eyre::eyre;
use color_eyre::eyre::Result;

use std::sync::Arc;
use structopt::StructOpt;
use regex::Regex;

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

    tokio::spawn(proofing_system::reports_generator(timeline.clone(), opt.idx));

    read_commands(timeline.clone(), opt.idx, opt.server_url).await;

    Ok(())
}


async fn read_commands(timeline : Arc<Timeline>, idx : usize, server : Uri) {
    print_command_msg();

    let orep_pat = Regex::new(r"r [+]?(\d+)").unwrap();

    let mut reader = BufReader::new(io::stdin());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        reader.read_line(&mut buffer).await.unwrap(); // Trusting io (don know if it works with > )
        {
            if let Some(cap) = orep_pat.captures(buffer.trim_end()) {
                let epoch  = cap[1].parse::<usize>();
                if epoch.is_err() { print_command_msg(); continue; }
                let _x = reports::obtain_location_report(timeline.clone(), idx, epoch.unwrap(), server.clone()).await;
                // TODO deal with return
            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() { println!("To obtain a report use: r <epoch>"); }