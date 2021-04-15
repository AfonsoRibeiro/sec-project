use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign::{self, PublicKey, SecretKey};
use color_eyre::eyre::{Result, Context};

#[derive(Debug,Serialize,Deserialize)]
pub struct Proof {
    idx_req : usize,
    idx_ass : usize,
    epoch : usize,
    loc_ass : (usize,usize),
}

impl Proof {
    pub fn new(epoch : usize, idx_req : usize, idx_ass : usize, loc_ass : (usize, usize)) -> Proof {
        Proof {
            idx_req,
            idx_ass,
            epoch,
            loc_ass,
        }
    }

    pub fn epoch(&self) -> usize { self.epoch }
    pub fn idx_req(&self) -> usize { self.idx_req }
    pub fn idx_ass(&self) -> usize { self.idx_ass }
    pub fn loc_ass(&self) -> (usize, usize) { self.loc_ass }
}

pub fn sign_proof(oursk : &SecretKey, proof : Proof) -> Vec<u8>{

    let plaintext = serde_json::to_vec(&proof).unwrap();

    sign::sign(&plaintext, oursk)
}

pub fn verify_proof(theirpk : &PublicKey, ciphertest : &Vec<u8>) -> Result<Proof> {

    let decoded_proof = sign::verify(ciphertest, theirpk).expect("Failed to verify proof");

    let proof = serde_json::from_slice(&decoded_proof).wrap_err_with(|| format!("Failed to parse proof"))?;

    Ok(proof)
}