mod validating;

use structopt::StructOpt;
use color_eyre::eyre::Result;

#[derive(StructOpt)]
#[structopt(name = "Single Server", about = "(Highly) Dependable Location Tracker")]
struct Opt {

    #[structopt(name = "server", long, default_value = "[::1]:50051")]
    server_addr : String

}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Server starting");

    let opt = Opt::from_args();


    Ok(())
}
