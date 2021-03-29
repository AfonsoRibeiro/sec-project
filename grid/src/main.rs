mod grid;

use std::fs::File;
use structopt::StructOpt;
use color_eyre::eyre::Result;

#[derive(StructOpt)]
#[structopt(name = "Grid", about = "Creates a grid and a timeline so points can know locations")]
struct Opt {

    #[structopt(short, long, default_value = "10")]
    size : usize,

    #[structopt(short, long, default_value = "100")]
    points : usize,

    #[structopt(short, long, default_value = "100")]
    epochs : usize,

    #[structopt(short, long, default_value = "grid/grid.txt")]
    file : String
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();
    // let mut stdout = std::io::stdout();

    let file = File::create(opt.file)?;

    let timeline = grid::create_timeline(opt.size, opt.points, opt.epochs);
    serde_json::to_writer(file, &timeline)?;

    Ok(())
}