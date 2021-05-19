use std::{fs::File, io::{BufReader, BufWriter}};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use serde_derive::{Deserialize, Serialize};
use eyre::{eyre, Context};
use color_eyre::eyre::Result;
use sodiumoxide::crypto::secretbox::Nonce;

use atomicwrites::{AtomicFile, AllowOverwrite};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
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
        self.grid[pos_x][pos_y].write().unwrap().insert(idx);
    }

    fn get_users_at_location(&self, pos_x : usize, pos_y : usize) -> Vec<usize> {
        self.grid[pos_x][pos_y].read().unwrap().iter().map(|&idx| idx).collect()
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Timeline {
    routes : RwLock<HashMap<usize, RwLock<HashMap<usize, Report>>>>, //epoch -> user id -> location/report
    proofs : RwLock<HashMap<usize, RwLock<HashMap<usize, Vec<Vec<u8>> >>>>, // user -> epoch -> proofs_given
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
            proofs : RwLock::new(HashMap::new()),
            timeline : RwLock::new(vec![]),
            size,
            blacklist : RwLock::new(HashSet::new()),
            nonces : RwLock::new(HashMap::new()),
            ha_nonces : RwLock::new(HashSet::new()),
            filename,
        }
    }

    pub fn report_not_submitted_at_epoch(&self, epoch: usize, idx: usize) -> bool {
        if let Some(reports_epoch) = self.routes.read().unwrap().get(&epoch) {
            !reports_epoch.read().unwrap().contains_key(&idx)
        }else {
            true
        }
    }

    pub fn add_user_location_at_epoch(&self, epoch: usize, (pos_x, pos_y) : (usize, usize), idx: usize, report : Vec<u8>) -> Result<()>{
        if self.blacklist.read().unwrap().contains(&idx) {
            return Err(eyre!("Malicious user detected!"));
        }
        if !self.valid_pos(pos_x, pos_y){
            return Err(eyre!("Invalid position"));
        }
        {
            let report = Report::new((pos_x, pos_y), report);
            let mut routes = self.routes.write().unwrap();
            if let Some(user_pos) =  routes.get(&epoch) {
                let mut writable_user_pos = user_pos.write().unwrap();
                if let Some(user_pos) =  writable_user_pos.get(&idx) {
                    if user_pos.loc != (pos_x, pos_y) {
                        self.blacklist.write().unwrap().insert(idx);
                        return Err(eyre!("Two different positions submitted for the same epoch"));
                    }
                } else {
                    writable_user_pos.insert(idx,report);
                }
            } else {
                let mut users_loc = HashMap::new();
                users_loc.insert(idx, report);
                routes.insert(epoch, RwLock::new(users_loc));
            }
        }
        {
            let mut vec = self.timeline.write().map_err(|_| eyre!("Unable to write"))?;

            for _ in vec.len()..=epoch {
                vec.push(Grid::new_empty(self.size));
            }
        }
        let vec = self.timeline.read().map_err(|_| eyre!("Unable to read"))?;
        vec[epoch].add_user_location(pos_x, pos_y, idx);
        Ok(())
    }

    pub fn add_proofs(&self, proofs : Vec<(usize, usize, Vec<u8>)>) {
        let mut p = self.proofs.write().unwrap();
        for (idx, epoch, proof) in proofs.into_iter() {
            if let Some(u_proof) = p.get_mut(&idx) {
                let mut u_proof = u_proof.write().unwrap();
                if let Some(u_e_proof) = u_proof.get_mut(&epoch) {
                    u_e_proof.push(proof);
                } else {
                    u_proof.insert(epoch, vec![proof]);
                }

            } else {
                let mut u_e_proof = HashMap::new();
                u_e_proof.insert(epoch, vec![proof]);

                p.insert(idx, RwLock::new(u_e_proof));
            }
        }
    }

    pub fn get_proofs(&self, idx : usize, epochs : &HashSet<usize>) -> Vec<Vec<u8>> { // Assumes vec is a set
        let mut proofs = vec![];

        let u_proofs = self.proofs.read().unwrap();
        if let Some(u_proofs) = u_proofs.get(&idx) {
            for epoch in epochs {
                let e_proofs = u_proofs.read().unwrap();
                if let Some(e_proofs) = e_proofs.get(epoch) {
                    proofs.extend(e_proofs.iter().map(|proof| proof.clone()));
                }
            }
        }
        proofs
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

    pub fn get_users_at_epoch_at_location(&self, epoch: usize, (pos_x, pos_y) : (usize, usize)) -> Option<Vec<(usize, Vec<u8>)>> {
        let vec = self.timeline.read().unwrap();

        if vec.len() > epoch && self.valid_pos(pos_x, pos_y) {
            let mut idxs_reports = vec![];
            let epoch_map = self.routes.read().unwrap();
            let epoch_map = epoch_map.get(&epoch).unwrap().read().unwrap();
            for idx in vec[epoch].get_users_at_location(pos_x, pos_y) {
                if let Some(report) = epoch_map.get(&idx) {
                    idxs_reports.push((idx, report.report.clone()));
                } else {
                    println!("WHY ME")
                }
            }
            Some(idxs_reports)
        } else {
            None
        }
    }

    pub fn get_user_report_at_epoch(&self, epoch: usize, idx: usize) -> Option<Vec<u8>> {
        if let Some(user_loc ) = self.routes.read().unwrap().get(&epoch) {
            if let Some(position) = user_loc.read().unwrap().get(&idx){
                return Some(position.report.clone());
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

pub async fn save_storage(filename : &str, timeline : &Timeline) -> Result<()> {
    let atomic_file = AtomicFile::new(filename, AllowOverwrite);

    atomic_file.write(|f| serde_json::to_writer(BufWriter::new(f), timeline) )?;

    Ok(())
}

pub fn retrieve_storage(file_name : &str) -> Result<Timeline> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", file_name)
    )? )
}


#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::crypto::secretbox;

    const SIZE : usize = 10;
    const EPOCH : usize = 5;
    const FILENAME : &str = "/storage/storage.txt";
    const POS_X : usize = 3;
    const DIFF_POS_X : usize = 5;
    const POS_Y : usize = 5;
    const IDX : usize = 785;

    #[test]
    fn new_grid() {
        let grid = Grid::new_empty(SIZE);
        assert_eq!(SIZE, grid.size);

        for collum in grid.grid.iter() {
            for square in collum.iter() {
                assert_eq!(0, square.read().unwrap().len())
            }
        }
    }

    #[test]
    fn grid_add_user() {
        let grid = Grid::new_empty(SIZE);
        grid.add_user_location(POS_X, POS_Y, IDX);
        let users = grid.get_users_at_location(POS_X, POS_Y);
        assert_eq!(1, users.len());
        assert_eq!(IDX, users[0]);
    }

    #[test]
    fn build_timeline() {
        let timeline = Timeline::new(SIZE, FILENAME.to_string());
        assert_eq!(SIZE, timeline.size);
    }

    #[tokio::test]
    async fn save_retrive_timeline() {
        let storage = Timeline::new(SIZE, FILENAME.to_string());

        assert!(save_storage(FILENAME, &storage).await.is_ok());

        let retrieved_storage = retrieve_storage(FILENAME).unwrap();

        assert_eq!(SIZE, retrieved_storage.size);
    }

    #[test]
    fn add_user() {
        let storage = Timeline::new(SIZE, FILENAME.to_string());

        assert!(storage.add_user_location_at_epoch(EPOCH, (POS_X, POS_Y), IDX, "report".as_bytes().to_vec()).is_ok());

        let report = storage.get_user_report_at_epoch(EPOCH, IDX).unwrap();

        assert_eq!(report, "report".as_bytes().to_vec());

        let users = storage.get_users_at_epoch_at_location(EPOCH, (POS_X, POS_Y)).unwrap();

        assert_eq!(1, users.len());

        assert_eq!((IDX, "report".as_bytes().to_vec()) , users[0]);
    }

    #[test]
    fn add_user_out_of_bound() {
        let storage = Timeline::new(SIZE, FILENAME.to_string());

        assert!(storage.add_user_location_at_epoch(EPOCH, (SIZE, POS_Y), IDX, "report".as_bytes().to_vec()).is_err());
    }

    #[test]
    fn double_report_at_same_epoch_diff_pos() {
        let storage = Timeline::new(SIZE, FILENAME.to_string());

        assert!(storage.add_user_location_at_epoch(EPOCH, (POS_X, POS_Y), IDX, "report".as_bytes().to_vec()).is_ok());

        assert!(storage.add_user_location_at_epoch(EPOCH, (DIFF_POS_X, POS_Y), IDX, "report".as_bytes().to_vec()).is_err());
    }

    #[test]
    fn double_report_at_same_epoch_same_pos() {
        let storage = Timeline::new(SIZE, FILENAME.to_string());

        assert!(storage.add_user_location_at_epoch(EPOCH, (POS_X, POS_Y), IDX, "report".as_bytes().to_vec()).is_ok());

        assert!(storage.add_user_location_at_epoch(EPOCH, (POS_X, POS_Y), IDX, "report".as_bytes().to_vec()).is_ok());
    }

    #[test]
    fn test_nonce() {
        let nonce : secretbox::Nonce = secretbox::gen_nonce();

        let storage = Timeline::new(SIZE, FILENAME.to_string());

        assert!(storage.valid_nonce(IDX, &nonce));

        assert!(storage.add_nonce(IDX, nonce));

        assert!(!storage.valid_nonce(IDX, &nonce));

        assert!(!storage.add_nonce(IDX, nonce));
    }
}