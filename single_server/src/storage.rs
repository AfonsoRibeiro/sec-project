//    Vec < HashMap < usize, (Report, Report_Encry) > >
// Epochs            user  -> report, non_repo

//Timeline Vec     < Vec<Vec< hashset > >
//         Epoch ->    x  y ->  user

use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use eyre::eyre;
use color_eyre::eyre::{self, Result};
use std::convert::TryFrom;

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pos_x : usize,
    pos_y : usize,
}

// pos_x -> pos_y -> user_id
#[derive(Debug, Deserialize, Serialize)]
pub struct Grid {
    grid : Vec<Vec<Vec<usize>>>,
    size: usize
}
//epoch -> user id -> location 
#[derive(Debug, Deserialize, Serialize)]
pub struct Timeline {
    routes : HashMap<usize, HashMap<usize, Location>>,
    timeline : Vec<Grid>,
    epochs : usize, //is really necessary?
}

impl Grid {
    fn new(size: usize) -> Grid {
        Grid{
            grid : vec![],
            size : size 
        }
    }

    fn new_empty(size : usize) -> Grid {
        Grid {
            grid :  vec![vec![vec![]; size]; size], //change
            size : size
        }
    }

    fn add_user_location(&mut self, pos_x : u32, pos_y : u32, idx : usize) {
        match Grid::parse_valid_pos(pos_x, pos_y){
            Ok(pos) => { self.grid[pos.0][pos.1].push(idx) }, 
            Err(err) => {}  //do something 
        }
    }

    fn get_neighbours(&self, pos_x : u32, pos_y : u32, idx : usize) -> Vec<usize>{ 
        match Grid::parse_valid_pos(pos_x, pos_y){
            Ok(pos) => {
                let mut neighbours : Vec<usize> = vec![];
                
                let lower_x = if pos.0 == 0 {pos.0} else {pos.0-1};
                let lower_y = if pos.1 == 0 {pos.1} else {pos.1-1};
                let upper_x = if pos.0+1 == self.size {pos.0} else {pos.0+1};
                let upper_y = if pos.1+1 == self.size {pos.0} else {pos.1+1};

                for x in lower_x..=upper_x {
                    for y in lower_y..=upper_y {
                        neighbours.extend(self.grid[x][y].iter());
                    }
                }
                neighbours.retain(|&p| p != idx); // remove itself

                neighbours
            }, 

            Err(err) => { return vec![] } //do something
        }
    }

    fn get_users_at_location(&self, pos_x : u32, pos_y : u32) -> Vec<usize> {
        match Grid::parse_valid_pos(pos_x, pos_y){
            Ok(pos) => {self.grid[pos.0][pos.1].iter().map(|&idx| idx ).collect()}, 
            Err(err) => { return vec![] } //do something
    
        }
    }

    pub fn parse_valid_pos(x : u32, y : u32) -> Result<(usize, usize)> {
        let (res_x, res_y) = (usize::try_from(x), usize::try_from(y));
        if res_x.is_err() /* || check limits */ {
            return Err(eyre!("Not a valid x position."));
        }
        if res_y.is_err() /* |eyre| check limits */ {
            return Err(eyre!("Not a valid y position."));
        }
        Ok((res_x.unwrap(), res_y.unwrap()))
    }
}

impl Timeline {
    fn new() -> Timeline {
        Timeline {
            routes : HashMap::new(),
            timeline : vec![],
            epochs : 0,
        }
    }

    pub fn is_point() -> bool { 
        true
    }

    fn add_user_location() {}
}