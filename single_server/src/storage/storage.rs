use std::{fs::File, io::{BufReader, BufWriter}, sync::RwLock, usize};

use dashmap::{DashMap, DashSet};

use serde_derive::{Deserialize, Serialize};
use eyre::{eyre, Context};
use color_eyre::{eyre::Result, owo_colors::OwoColorize};

#[derive(Debug)]
struct Report {
    loc : (usize, usize),
    //report : bitvector
} 

// pos_x -> pos_y -> user_id
#[derive(Debug)]
struct Grid {
    grid : Vec<Vec<DashSet<usize>>>,
    size: usize
}

impl Grid {

    fn new_empty(size : usize) -> Grid {
        Grid {
            grid : (0..size).map(|_| (0..size).map(|_| DashSet::new()).collect()).collect(),
            size
        }
    }

    fn add_user_location(&self, pos_x : usize, pos_y : usize, idx : usize) {
        self.grid[pos_x][pos_y].insert(idx);
    }

    fn get_neighbours(&self, pos_x : usize, pos_y : usize, idx : usize) -> Vec<usize>{
        let mut neighbours : Vec<usize> = vec![];

        let lower_x = if pos_x == 0 {pos_x} else {pos_x-1};
        let lower_y = if pos_y == 0 {pos_y} else {pos_y-1};
        let upper_x = if pos_x+1 == self.size {pos_x} else {pos_x+1};
        let upper_y = if pos_y+1 == self.size {pos_y} else {pos_y+1};

        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                for id in self.grid[x][y].iter(){
                    neighbours.push(id.clone());
                }
            }
        }
        neighbours.retain(|&p| p != idx); // remove itself

        neighbours
    }

    fn get_users_at_location(&self, pos_x : usize, pos_y : usize) ->Vec<usize> {
        self.grid[pos_x][pos_y].iter().map(|idx| idx.clone()).collect()
    }
}

//epoch -> user id -> location
#[derive(Debug)] 
pub struct Timeline {
    routes : DashMap<usize, DashMap<usize, (usize, usize)>>,
    timeline : RwLock<Vec<Grid>>,
    size: usize,
    blacklist: DashSet<usize>
}

impl Timeline {
    pub fn new(size : usize) -> Timeline {
        Timeline {
            routes : DashMap::new(),
            timeline : RwLock::new(vec![]),
            size,
            blacklist : DashSet::new(),
        }
    }

    pub fn add_user_location_at_epoch(&self, epoch: usize, pos_x : usize, pos_y : usize, idx: usize) -> Result<()>{ //TODO: check if it is valid -> report
        if self.blacklist.contains(&idx) {
            return Err(eyre!("Malicious user detected!"));
        }
        if let Some(user_pos) =  self.routes.get_mut(&epoch) {
            if let Some(_) = user_pos.insert(idx,(pos_x, pos_y)) {
                user_pos.remove(&idx);
                self.blacklist.insert(idx);
                return Err(eyre!("Two positions submitted for the same epoch"));
            }

        }else { //RwLock bc of insert
           let users_loc= DashMap::new();
           users_loc.insert(idx, (pos_x, pos_y));
           self.routes.insert(epoch, users_loc);
        }
        let mut vec = self.timeline.write().unwrap(); // Fix this : dont assume this

        for epoch_value in vec.len()..=epoch {
            vec.push(Grid::new_empty(self.size));
        }
        vec[epoch].add_user_location(pos_x, pos_y, idx);
        Ok(())
    }

    pub fn valid_pos(&self, x : usize, y : usize) -> bool {
        x < self.size && y < self.size
    }

    pub fn valid_neighbour(&self, x : usize, y : usize) -> ((usize, usize),(usize, usize)) {
        let lower_x = if x == 0 {x} else {x-1};
        let lower_y = if y == 0 {y} else {y-1};
        let upper_x = if x+1 == self.size {x} else {x+1};
        let upper_y = if y+1 == self.size {y} else {y+1};
        ((lower_x, lower_y), (upper_x, upper_y))
    }

    pub fn get_users_at_epoch_at_location(&self, epoch: usize, pos_x : usize, pos_y : usize) -> Option<Vec<usize>> {
        let vec = self.timeline.read().unwrap();  // Fix this : dont assume this

        if vec.len() > epoch && self.valid_pos(pos_x, pos_y){
            Some(vec[epoch].get_users_at_location(pos_x,pos_y))
        }else {
            None
        }
    }

    pub fn get_user_location_at_epoch(&self, epoch: usize, idx: usize) -> Option<(usize, usize)> {
        if let Some(user_loc ) = self.routes.get(&epoch) {
            if let Some(position) = user_loc.get(&idx){
                let (x, y) = position.value();
                return Some((*x,*y));
            }
        }
        None
    }
}


// pub fn save_storage(file_name : &str, timeline : &Timeline) -> Result<()> { //TODO: make it async, depends on how the database is updated
//     let file = File::create(file_name)?;

//     serde_json::to_writer(BufWriter::new(file), timeline)?;

//     Ok(())
// }

// pub fn retrieve_storage(file_name : &str) -> Result<Timeline> {
//     let file = File::open(file_name)?;
//     let reader = BufReader::new(file);

//     Ok(serde_json::from_reader(reader).wrap_err_with(
//         || format!("Failed to parse struct Timeline from file '{:}'", file_name)
//     )? )
// }