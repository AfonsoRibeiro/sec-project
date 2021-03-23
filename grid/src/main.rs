fn parse_args(args: &[String]) -> (u32, u32, u32) {
    if args.len() != 4 {
        println!("Error: Argument must be 3, [size of grid] [number of points] [number of ephocs]");
        std::process::exit(1);
    }

    (args[1].parse().unwrap(), args[2].parse().unwrap(), args[3].parse().unwrap())
}

struct Grid {
    grid : Vec<Vec<i32>>
}

// impl Grid {
//     fn new(size : u32) -> Grid {
//         { grid = vec}
//     }
// }

struct Timeline {
    timeline : Vec<Grid>
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (size, points, ephocs) = parse_args(&args);


}

fn create_grid() {

}

