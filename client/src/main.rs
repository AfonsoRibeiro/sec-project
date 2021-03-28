
use structopt::StructOpt;
use color_eyre::eyre::Result;

// use locationstorage::greeter_client::GreeterClient;
// use locationstorage::HelloRequest;

// pub mod hello_world {
//     tonic::include_proto!("locationstorage");
// }

#[derive(StructOpt)]
#[structopt(name = "Client", about = "Reporting and verifying locations since 99.")]
struct Opt {
    #[structopt(name = "server", long)]
    server_url : String, // TODO
    #[structopt(long)]
    idx : usize,
    #[structopt(name = "file", long)]
    grid_file : String
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();


    Ok(())

}
