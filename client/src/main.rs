mod proofing_system;
mod reports;

use std::{thread, time};

use structopt::StructOpt;
use std::sync::Arc;
use color_eyre::eyre::Result;

use grid::grid::{Timeline, retrieve_timeline};

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {

    #[structopt(name = "server", long, default_value = "http://[::1]:50051")]
    server_url : String,

    #[structopt(name = "id", long)]
    idx : usize,

    #[structopt(name = "grid", long, default_value = "grid/grid.txt")]
    grid_file : String
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Starting");

    let opt = Opt::from_args();

    let timeline = Arc::new(retrieve_timeline(&opt.grid_file)?);

    // TODO check if idx in grid

    tokio::spawn(proofing_system::start_proofer(opt.idx, timeline.clone()));

    thread::sleep(time::Duration::from_millis(1000));

    for epoch in 0..timeline.epochs() {
        println!("EPOCH: {:}", epoch);
        match timeline.get_neighbours_at_epoch(opt.idx, epoch) {
            Some(neighbours) => {
                #[allow(unused_must_use)]
                for id_dest in neighbours {
                    println!("Getting proof from {:}", id_dest);
                    proofing_system::request_location_proof(opt.idx, epoch, id_dest); // TODO should not just end procces FIX
                }
                // TODO : Wait for responses
            }
            None => panic!("Idx from args doens't exist in grid.") // Should never happen
        }

        thread::sleep(time::Duration::from_millis(2000));
    }

    Ok(())
}
