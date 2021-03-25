mod grid;

use color_eyre::eyre::Result;

fn parse_args(args: &[String]) -> Result<(usize, usize, usize), std::num::ParseIntError> {

    assert_eq!(args.len(), 4, "Error: Argument must be 3, [size of grid : usize] [number of points : usize] [number of ephocs : usize]");

    Ok((args[1].parse()?, args[2].parse()?, args[3].parse()?))
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Vec<String> = std::env::args().collect();

    match parse_args(&args) {
        Ok((size, points, epochs)) => {
            let timeline = grid::create_timeline(size, points, epochs);
            println!("Timeline = {:?}", timeline);
        }
        Err(_) => panic!("Error : One of the arguments wasn't a valid usize")
    }

    Ok(())
}
