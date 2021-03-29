mod client_client_comunication;
mod client_server_comunication;

use structopt::StructOpt;
use color_eyre::eyre::Result;

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {
    #[structopt(name = "server", long, default_value="http://[::1]:50051")]
    server_url : String, // TODO
    #[structopt(long)]
    idx : usize,
    #[structopt(name = "grid", long, default_value="grid.txt")]
    grid_file : String
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    Ok(())
}
