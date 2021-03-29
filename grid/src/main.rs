mod grid;

use structopt::StructOpt;
use color_eyre::eyre::Result;

#[derive(StructOpt)]
#[structopt(name = "Grid", about = "Values to create the timeline.")]
struct Opt {
    #[structopt(short, long, default_value = "10")]
    size : usize,
    #[structopt(short, long, default_value = "100")]
    points : usize,
    #[structopt(short, long, default_value = "100")]
    epochs : usize,
    // #[structopt(short, long, default_value = "100")]
    // file : String
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();
    // let mut stdout = std::io::stdout();

    let timeline = grid::create_timeline(opt.size, opt.points, opt.epochs);
    println!("Timeline = {:?}", timeline);

    Ok(())
}