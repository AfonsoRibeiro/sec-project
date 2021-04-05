//    Vec < HashMap < usize, (Report, Report_Encry) > >
// Epochs            user  -> report, non_repo

//Timeline Vec     < Vec<Vec< hashset > >
//         Epoch ->    x  y ->  user

use std::{collections::{HashMap, HashSet}, fs::File, io::{BufReader, BufWriter}};

use serde_derive::{Deserialize, Serialize};
use eyre::Context;
use color_eyre::eyre::{self, Result};


// pos_x -> pos_y -> user_id
#[derive(Debug, Deserialize, Serialize)]
struct Grid {
    grid : Vec<Vec<HashSet<usize>>>,
    size: usize
}
//epoch -> user id -> location
#[derive(Debug, Deserialize, Serialize)]
pub struct Timeline {
    routes : HashMap<usize, HashMap<usize, (usize, usize)>>,
    timeline : Vec<Grid>,
    size: usize,
}

impl Grid {

    fn new_empty(size : usize) -> Grid {
        Grid {
            grid : (0..size).map(|_| (0..size).map(|_| HashSet::new()).collect()).collect(),
            size
        }
    }

    fn add_user_location(&mut self, pos_x : usize, pos_y : usize, idx : usize) {
        self.grid[pos_x][pos_y].insert(idx);
    }

    fn get_neighbours(&self, pos_x : usize, pos_y : usize, idx : usize) -> Vec<usize>{
        let mut neighbours : Vec<usize> = vec![];

        let lower_x = if pos_x == 0 {pos_x} else {pos_x-1};
        let lower_y = if pos_y == 0 {pos_y} else {pos_y-1};
        let upper_x = if pos_x+1 == self.size {pos_x} else {pos_x+1};
        let upper_y = if pos_y+1 == self.size {pos_x} else {pos_y+1};

        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                neighbours.extend(self.grid[x][y].iter());
            }
        }
        neighbours.retain(|&p| p != idx); // remove itself

        neighbours
    }

    fn get_users_at_location(&self, pos_x : usize, pos_y : usize) ->Vec<usize> {
        self.grid[pos_x][pos_y].iter().map(|&idx| idx).collect()
    }

}

impl Timeline {
    pub fn new(size : usize) -> Timeline {
        Timeline {
            routes : HashMap::new(),
            timeline : vec![],
            size
        }
    }

    pub fn add_user_location_at_epoch(&mut self, epoch: usize, pos_x : usize, pos_y : usize, idx: usize) { //TODO: check if it is valid -> report
        if let Some(user_pos) =  self.routes.get_mut(&epoch) {
            user_pos.insert(idx,(pos_x, pos_y));

        }else {
           let mut users_loc= HashMap::new();
           users_loc.insert(idx, (pos_x, pos_y));
           self.routes.insert(epoch, users_loc);
        }

        for epoch_value in self.timeline.len()..=epoch {
            self.timeline.push(Grid::new_empty(self.size));
        }
        self.timeline[epoch].add_user_location(pos_x, pos_y, idx);
    }

    pub fn valid_pos(&self, x : usize, y : usize) -> bool {
        x < self.size && y < self.size
    }

    pub fn get_users_at_epoch_at_location(&self, epoch: usize, pos_x : usize, pos_y : usize) -> Option<Vec<usize>> {
        if self.timeline.len() > epoch && self.valid_pos(pos_x, pos_y){
            Some(self.timeline[epoch].get_users_at_location(pos_x,pos_y))
        }else {
            None
        }
    }

    pub fn get_user_location_at_epoch(&self, epoch: usize, idx: usize) -> Option<(usize, usize)> {
        if let Some(user_loc ) = self.routes.get(&epoch) {
            if let Some((x, y)) = user_loc.get(&idx) {
                return Some((*x,*y));
            }
        }
        None
    }
}

pub fn save_storage(file_name : &str, timeline : &Timeline) -> Result<()> { //TODO: make it async, depends on how the database is updated
    let file = File::create(file_name)?;

    serde_json::to_writer(BufWriter::new(file), timeline)?;

    Ok(())
}

pub fn retrieve_storage(file_name : &str) -> Result<Timeline> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", file_name)
    )? )
}