use rand::Rng;
use std::{cmp::min, fs::File};
use std::io::{BufReader, BufWriter};
use std::collections::{HashSet, HashMap};
use color_eyre::eyre::{Context, Result};
use serde_derive::{Deserialize, Serialize};

// Grid simulated thru a single vector
#[derive(Debug, Deserialize, Serialize, PartialEq)]
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
        let upper_y = if y+1 == self.size {y} else {y+1};

        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                neighbours.extend( self.grid[self.get_index(x, y)].iter() );
            }
        }
        neighbours.retain(|&p| p != point); // remove itself

        neighbours
    }

    fn get_position(&self, index : usize) -> (usize, usize) {
        (index % self.size, index / self.size)
    }

    fn get_index(&self, x : usize, y : usize) -> usize {
        x + self.size * y
    }

    fn min_neighbours(&self) -> usize {
        let mut min_n = usize::MAX;

        for i in 0..self.size {
            for j in 0..self.size {
                let lower_x = if i == 0 {i} else {i-1};
                let lower_y = if j == 0 {j} else {j-1};
                let upper_x = if i+1 == self.size {i} else {i+1};
                let upper_y = if j+1 == self.size {j} else {j+1};

                let mut n = 0_usize;

                for x in lower_x..=upper_x {
                    for y in lower_y..=upper_y {
                        n += self.grid[self.get_index(x, y)].len();
                    }
                }

                n -= 1;

                min_n = min(min_n, n);
            }
        }

        min_n
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Timeline {
    timeline : Vec<Grid>,

    // Mapping between point and their route
    routes : HashMap<usize, Vec<usize>>, // For each point where it is in each epoch

    epochs : usize,

    size : usize,

    pub f_line : usize,
}

impl Timeline {
    fn new(size : usize) -> Timeline {
        Timeline {
            timeline : vec![],
            routes : HashMap::new(),
            epochs : 0,
            size,
            f_line : usize::MAX,
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

        self.f_line = min(self.f_line, (new_grid.min_neighbours()-1) / 2);
        self.timeline.push(new_grid);

        self.epochs += 1;
    }

    pub fn create_timeline(size : usize, points : usize, epochs : usize) -> Timeline {
        let mut timeline = Timeline::new(size);
        for _ in 0..epochs {
            timeline.add_epoch(Grid::new_randomly_filled(size, points));
        }
        timeline
    }

    pub fn epochs(&self) -> usize { self.epochs }

    pub fn is_point(&self, point : usize) -> bool { self.routes.contains_key(&point) }

    pub fn valid_pos(&self, x : usize, y : usize) -> bool {
        x < self.size && y < self.size
    }

    pub fn get_neighbours_at_epoch(&self, point : usize, epoch : usize) -> Option<Vec<usize>> {
        if epoch >= self.epochs || !self.routes.contains_key(&point) { return None; }

        let index = self.routes[&point][epoch];

        Some( self.timeline[epoch].get_neighbours(index, point) )
    }

    pub fn get_index_at_epoch(&self, point : usize, epoch : usize) -> Option<usize> {
        if epoch >= self.epochs || !self.routes.contains_key(&point) { return None; }

        Some(self.routes[&point][epoch])
    }

    fn get_position(&self, index : usize) -> (usize, usize) {
        (index % self.size, index / self.size)
    }

    pub fn get_location_at_epoch(&self, point : usize, epoch : usize) -> Option<(usize, usize)> {
        if epoch >= self.epochs || !self.routes.contains_key(&point) { return None; }

        Some( self.get_position( self.routes[&point][epoch] ) )
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

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE : usize = 10;
    const POINTS : usize = 1000;
    const EPOCHS : usize = 10;
    const PATH : &str = "grid.txt";

    #[test]
    fn new_grid() {
        let grid = Grid::new_empty(SIZE);
        assert_eq!(SIZE, grid.size);
        assert_eq!(SIZE*SIZE, grid.total_size);

        for square in grid.grid.iter() {
            assert_eq!(0, square.len())
        }
    }

    #[test]
    fn new_filled_grid() {
        let grid = Grid::new_randomly_filled(SIZE, POINTS);

        assert_eq!(SIZE, grid.size);
        assert_eq!(SIZE*SIZE, grid.total_size);

        let mut n_points = 0;
        for square in grid.grid.iter() {
            n_points += square.len();
        }
        assert_eq!(POINTS, n_points)
    }

    #[test]
    fn get_position() {
        let grid = Grid::new_randomly_filled(SIZE, POINTS);

        let index = SIZE + 5;

        let (x, y) = grid.get_position(index);

        assert_eq!(5, x);
        assert_eq!(1, y);

        assert_eq!(index, grid.get_index(x, y));
    }

    #[test]
    fn build_timeline() {
        let timeline = Timeline::create_timeline(SIZE, POINTS, EPOCHS);

        assert_eq!(SIZE, timeline.size);

        assert_eq!(EPOCHS, timeline.epochs());
    }

    #[test]
    fn is_point() {
        let timeline = Timeline::create_timeline(SIZE, POINTS, EPOCHS);
        assert!(timeline.is_point(POINTS - POINTS/2));
        assert!(!timeline.is_point(POINTS + POINTS/2));
    }

    #[test]
    fn save_retrive_timeline() {
        let timeline = Timeline::create_timeline(SIZE, POINTS, EPOCHS);

        assert!(save_timeline(PATH, &timeline).is_ok());

        let retrieved_timeline = retrieve_timeline(PATH);

        assert!(retrieved_timeline.is_ok());

        assert_eq!(timeline, retrieved_timeline.unwrap());
    }
}