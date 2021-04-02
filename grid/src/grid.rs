use std::{fs::File, usize};
use std::io::{BufReader, BufWriter};
use rand::Rng;
use eyre::eyre;
use std::collections::{HashSet, HashMap};
use std::convert::TryFrom;
use color_eyre::eyre::{Context, Result};

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
            size,
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

        let (x, y) = self.get_position(index);

        let lower_x = if x == 0 {x} else {x-1};
        let lower_y = if y == 0 {y} else {y-1};
        let upper_x = if x+1 == self.size {x} else {x+1};
        let upper_y = if y+1 == self.size {x} else {y+1};

        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                neighbours.extend( self.grid[self.get_index(x, y)].iter() );
            }
        }
        neighbours.retain(|&p| p != point); // remove itself

        neighbours
    }

    pub fn get_position(&self, index : usize) -> (usize, usize) {
        (index % self.size, index / self.size)
    }

    fn get_index(&self, x : usize, y : usize) -> usize {
        x + self.size * y
    }

    // fn find_point(&self, point : usize) -> Option<(usize, usize)> {
    //     for (index, pos) in self.grid.iter().enumerate() {
    //         if pos.contains(&point) {
    //             return Some(self.get_position(index));
    //         }
    //     }
    //     None
    // }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Timeline {
    timeline : Vec<Grid>,

    // Mapping between point and their route
    routes : HashMap<usize, Vec<usize>>, // For each point where it is in each epoch

    epochs : usize,
}

impl Timeline {
    fn new() -> Timeline {
        Timeline {
            timeline : vec![],
            routes : HashMap::new(),
            epochs : 0,
        }
    }

    fn add_epoch(&mut self, new_grid : Grid) {
        new_grid.grid.iter().enumerate().for_each( |(index, square)|
            square.iter().for_each( |point|
                match self.routes.get_mut(point) {
                    Some(route) => { route.push(index); }
                    None => { self.routes.insert(*point, vec![index]); }
                }
            )
        );
        self.timeline.push(new_grid);

        self.epochs += 1;
    }

    pub fn epochs(&self) -> usize { self.epochs }

    pub fn is_point(&self, point : usize) -> bool { self.routes.contains_key(&point) }

    pub fn get_neighbours_at_epoch(&self, point : usize, epoch : usize) -> Option<Vec<usize>> {
        if epoch >= self.epochs || !self.routes.contains_key(&point) { return None; }
    
        let index = self.routes[&point][epoch];
    
        Some( self.timeline[epoch].get_neighbours(index, point) )
    }
    
    pub fn get_index_at_epoch(&self, point : usize, epoch : usize) -> Option<usize> {
        if epoch >= self.epochs { return None; }

        Some(self.routes[&point][epoch])
    }

    pub fn create_timeline(size : usize, points : usize, epochs : usize) -> Timeline {
        let mut timeline = Timeline::new();
        for _ in 0..epochs {
            timeline.add_epoch(Grid::new_randomly_filled(size, points));
        }
        timeline
    }

    pub fn parse_valid_pos(x : u32, y : u32) -> Result<(usize, usize)> {
        let (res_x, res_y) = (usize::try_from(x), usize::try_from(y));
        if res_x.is_err() /* || check limits */ {
            return Err(eyre!("Not a valid x position."));
        }
        if res_y.is_err() /* || check limits */ {
            return Err(eyre!("Not a valid y position."));
        }
        Ok((res_x.unwrap(), res_y.unwrap()))
    }
}

// Needs to be safe!

pub fn save_timeline(file_name : &str, timeline : &Timeline) -> Result<()> {
    let file = File::create(file_name)?;

    serde_json::to_writer(BufWriter::new(file), timeline)?;

    Ok(())
}

pub fn retrieve_timeline(file_name : &str) -> Result<Timeline> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", file_name)
    )? )
}