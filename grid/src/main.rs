fn parse_args(args: &[String]) -> Result<(u32, u32, u32), std::num::ParseIntError> {

    assert_eq!(args.len(), 4, "Error: Argument must be 3, [size of grid : u32] [number of points : u32] [number of ephocs : u32]");

    Ok((args[1].parse::<u32>()?, args[2].parse()?, args[3].parse()?))
}

#[derive(Debug)]
struct Grid {
    grid : Vec<Vec<i32>>
}

// impl Grid {
//     fn new(size : u32) -> Grid {
//         { grid = vec}
//     }
// }

#[derive(Debug)]
struct Timeline {
    timeline : Vec<Grid>
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match parse_args(&args) {
        Ok((size, points, epochs)) => create_grid(size, points, epochs),
        Err(_) => panic!("Error : One of the arguments wasn't a valid u32")
    }



}

fn create_grid(size : u32, points : u32, ephocs : u32) {

}

