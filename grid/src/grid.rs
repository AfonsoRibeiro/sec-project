use std::collections::HashSet;
use rand::Rng;


// Grid simulated thru a single vector
#[derive(Debug)]
struct Grid {
    grid : Vec<HashSet<usize>>,
    total_size : usize,
    size : usize,
}

impl Grid {
    fn new_empty(size : usize) -> Grid {
        Grid {
            grid : (0..size*size).map(|_| HashSet::new()).collect(),
            total_size : size*size,
            size : size,
        }
    }
    fn new_randomly_filled(size : usize, points : usize) -> Grid {
        let mut rng = rand::thread_rng();
        let mut grid = Grid::new_empty(size);
        for i in 0..points {
            grid.grid[rng.gen_range(0..grid.total_size)].insert(i);
        }
        grid
    }
}

#[derive(Debug)]
pub struct Timeline {
    timeline : Vec<Grid>,
    ephocs : usize,
}

impl Timeline {
    fn new() -> Timeline {
        Timeline {
            timeline : vec![],
            ephocs : 0,
        }
    }

    fn add_epoch(&mut self, new_grid : Grid) {
        self.timeline.push(new_grid);
        self.ephocs += 1;
    }
}

pub fn create_timeline(size : usize, points : usize, ephocs : usize) -> Timeline {
    let mut timeline = Timeline::new();
    for _ in 0..ephocs {
        timeline.add_epoch(Grid::new_randomly_filled(size, points));
    }
    timeline
}