use structopt::StructOpt;
use color_eyre::eyre::Result;

#[derive(StructOpt)]
#[structopt(name = "Security", about = "Creates the key pair")]
struct Opt {

    #[structopt(long, default_value = "20")]
    n_clients : usize,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();



    Ok(())
}