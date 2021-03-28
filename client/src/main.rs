use structopt::StructOpt;
use color_eyre::eyre::Result;

use protos::location_storage::HelloRequest;
use protos::location_storage::greeter_client::GreeterClient;

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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    //let opt = Opt::from_args();

    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);


    Ok(())

}
