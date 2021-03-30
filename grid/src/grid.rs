use std::fs::File;
use std::io::{BufReader, BufWriter};
use rand::Rng;
use std::collections::{HashSet, HashMap};

use color_eyre::eyre::Result;

use serde_derive::{Deserialize, Serialize};

// Grid simulated thru a single vector
#[derive(Debug, Deserialize, Serialize)]
pub struct Grid {
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

    fn get_neighbours(&self, index : usize, point : usize) -> Vec<usize> {
        let mut neighbours : Vec<usize> = vec![];

        neighbours.extend(self.grid[index].iter());
        neighbours.retain(|&p| p != point);

        //TODO add rest of neighbours

        neighbours
    }

    fn get_position(&self, index : usize) -> (usize, usize) {
        (index % self.total_size, index / self.total_size)
    }

    fn get_index(&self, x : usize, y : usize) -> usize {
        x + self.size * y
    }

    fn find_point(&self, point : usize) -> Option<(usize, usize)> {
        for (index, pos) in self.grid.iter().enumerate() {
            if pos.contains(&point) {
                return Some(self.get_position(index));
            }
        }
        None
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Timeline {
    timeline : Vec<Grid>,
    wisw : HashMap<usize, HashSet<usize>>,
    ephocs : usize,
}

impl Timeline {
    fn new() -> Timeline {
        Timeline {
            timeline : vec![],
            wisw : HashMap::new(),
            ephocs : 0,
        }
    }

    fn add_epoch(&mut self, new_grid : Grid) {
        self.timeline.push(new_grid);
        // TODO where is who
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

pub fn save_timeline(file_name : &str, timeline : &Timeline) -> Result<()> {
    let file = File::create(file_name)?;

    serde_json::to_writer(BufWriter::new(file), timeline)?;

    Ok(())
}

pub fn retrieve_timeline(file_name : &str) -> Result<Timeline> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}
