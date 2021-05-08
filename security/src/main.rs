mod key_management;

use structopt::StructOpt;
use color_eyre::eyre::Result;

#[derive(StructOpt)]
#[structopt(name = "Security", about = "Creates the key pair")]
struct Opt {

    #[structopt(name = "clients", long, default_value = "20")]
    n_clients : usize,

    #[structopt(name = "servers", long, default_value = "5")]
    n_servers : usize,

    #[structopt(name = "keys", long, default_value = "security/keys")]
    keys_dir : String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    key_management::save_keys(opt.n_clients, opt.n_servers, opt.keys_dir)
}