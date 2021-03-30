mod proofing_system;
mod reporting;

use structopt::StructOpt;
use color_eyre::eyre::Result;

use grid::grid::retrieve_timeline;

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

    let opt = Opt::from_args();

    let timeline = retrieve_timeline(&opt.grid_file)?;

    proofing_system::start_proofer(opt.idx, timeline).await?;

    // for epoch in 0..timeline.epochs() {

    // }

    Ok(())
}
