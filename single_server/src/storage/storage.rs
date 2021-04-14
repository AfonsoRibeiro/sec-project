use std::{fs::File, io::{BufReader, BufWriter}};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use serde_derive::{Deserialize, Serialize};
use eyre::{eyre, Context};
use color_eyre::eyre::Result;
use sodiumoxide::crypto::secretbox::Nonce;

#[derive(Debug, Serialize, Deserialize)]
struct Report {
    loc : (usize, usize),
    report : Vec<u8>
}

impl Report {

    fn new(loc: (usize, usize), report: Vec<u8>) -> Report {
        Report {
            loc,
            report
        }
    }
}
// pos_x -> pos_y -> user_id
#[derive(Debug, Serialize, Deserialize)]
struct Grid {
    grid : Vec<Vec<RwLock<HashSet<usize>>>>,
    size: usize
}

impl Grid {

    fn new_empty(size : usize) -> Grid {
        Grid {
            grid : (0..size).map(|_| (0..size).map(|_| RwLock::new(HashSet::new()) ).collect()).collect(),
            size
        }
    }

    fn add_user_location(&self, pos_x : usize, pos_y : usize, idx : usize) {
        self.grid[pos_x][pos_y].write().unwrap().insert(idx); //fix unwrap
    }

    fn get_users_at_location(&self, pos_x : usize, pos_y : usize) ->Vec<usize> {
        self.grid[pos_x][pos_y].read().unwrap().iter().map(|&idx| idx).collect() //fix unwrap
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Timeline {
    routes : RwLock<HashMap<usize, RwLock<HashMap<usize, Report>>>>, //epoch -> user id -> location
    timeline : RwLock<Vec<Grid>>,
    size : usize,
    blacklist : RwLock<HashSet<usize>>,
    nonces : RwLock<HashMap<usize, HashSet<Nonce>>>,
    ha_nonces : RwLock<HashSet<Nonce>>,
    filename: String,
}

impl Timeline {
    pub fn new(size : usize, filename: String) -> Timeline {
        Timeline {
            routes : RwLock::new(HashMap::new()),
            timeline : RwLock::new(vec![]),
            size,
            blacklist : RwLock::new(HashSet::new()),
            nonces : RwLock::new(HashMap::new()),
            ha_nonces : RwLock::new(HashSet::new()),
            filename,
        }
    }

    pub fn add_user_location_at_epoch(&self, epoch: usize, (pos_x, pos_y) : (usize, usize), idx: usize, report : Vec<u8>) -> Result<()>{ //TODO: check if it is valid -> report
        if self.blacklist.read().unwrap().contains(&idx) {
            return Err(eyre!("Malicious user detected!"));
        }
        {
            let mut routes = self.routes.write().unwrap();
            let report = Report::new((pos_x, pos_y), report);
            if let Some(user_pos) =  routes.get(&epoch) {
                if let Some(report) = user_pos.write().unwrap().insert(idx,report) {
                    if report.loc != (pos_x, pos_y) {
                        user_pos.write().unwrap().remove(&idx);
                        self.blacklist.write().unwrap().insert(idx);
                        return Err(eyre!("Two positions submitted for the same epoch"));
                    } else {
                        return Ok(()); //client resubmited report, no problem
                    }
                }

            } else {
                let mut users_loc = HashMap::new();
                users_loc.insert(idx, report);
                routes.insert(epoch, RwLock::new(users_loc));
            }
        }
        {
            let mut vec = self.timeline.write().unwrap(); // Fix this : dont assume this

            for _ in vec.len()..=epoch {
                vec.push(Grid::new_empty(self.size));
            }
        }
        let vec = self.timeline.read().unwrap();
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
        } else {
            None
        }
    }

    pub fn get_user_location_at_epoch(&self, epoch: usize, idx: usize) -> Option<(usize, usize)> {
        if let Some(user_loc ) = self.routes.read().unwrap().get(&epoch) {
            if let Some(position) = user_loc.read().unwrap().get(&idx){
                return Some(position.loc);
            }
        }
        None
    }

    pub fn valid_nonce(&self, idx : usize, nonce : &Nonce) -> bool {
        let nonces = self.nonces.read().unwrap();
        if let Some(user_nonces) = nonces.get(&idx) {
            !user_nonces.contains(&nonce)
        } else {
            true
        }
    }

    pub fn add_nonce(&self, idx : usize, nonce : Nonce) -> bool {
        let mut nonces = self.nonces.write().unwrap();
        if let Some(user_nonces) = nonces.get_mut(&idx){
            user_nonces.insert(nonce)
        } else { //RwLock bc of insert (before if else)
           let mut user_nonce= HashSet::new();
           user_nonce.insert(nonce);
           nonces.insert(idx, user_nonce);
           true
        }
    }

    pub fn valid_ha_nonce(&self, nonce : &Nonce) -> bool {
        let nonces = self.ha_nonces.read().unwrap();
        !nonces.contains(&nonce)
    }

    pub fn add_ha_nonce(&self, nonce : Nonce) -> bool {
        let mut nonces = self.ha_nonces.write().unwrap();
        nonces.insert(nonce)
    }

    pub fn filename(&self) -> &str { &self.filename }
}

pub async fn save_storage(filename : &str, timeline : &Timeline) -> Result<()> { //TODO: make it async, depends on how the database is updated
    let file = File::create(filename)?;

    serde_json::to_writer(BufWriter::new(file), &timeline)?;

    Ok(())
}

pub fn retrieve_storage(file_name : &str) -> Result<Timeline> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", file_name)
    )? )
}