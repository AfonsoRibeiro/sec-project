use structopt::StructOpt;
use color_eyre::eyre::Result;

use grid::grid::create_timeline; //::create_timeline;
use grid::grid::save_timeline; //::save_timeline;

#[derive(StructOpt)]
#[structopt(name = "Grid", about = "Creates a grid and a timeline so points can know locations")]
struct Opt {

    #[structopt(short, long, default_value = "5")]
    size : usize,

    #[structopt(short, long, default_value = "100")]
    points : usize,

    #[structopt(short, long, default_value = "3")]
    epochs : usize,

    #[structopt(short, long, default_value = "grid/grid.txt")]
    file : String
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::from_args();

    let timeline = create_timeline(opt.size, opt.points, opt.epochs);

    save_timeline(&opt.file, &timeline)
}