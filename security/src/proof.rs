use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign::{self, PublicKey, SecretKey};
use color_eyre::eyre::{Result, Context};
use eyre::eyre;

#[derive(Debug,Serialize,Deserialize)]
pub struct Proof {
    idx_req : usize,
    idx_ass : usize,
    epoch : usize,
    loc_req : (usize,usize),
}

impl Proof {
    pub fn new(epoch : usize, idx_req : usize, idx_ass : usize, loc_req : (usize, usize)) -> Proof {
        Proof {
            idx_req,
            idx_ass,
            epoch,
            loc_req,
        }
    }
    pub fn loc_req(&self) -> (usize, usize) {
        self.loc_req
    }
}

pub fn sign_proof(oursk : &SecretKey, proof : Proof) -> Vec<u8>{

    let plaintext = serde_json::to_vec(&proof).unwrap();

    sign::sign(&plaintext, oursk)
}

pub fn verify_proof(theirpk : &PublicKey, ciphertest : &Vec<u8>) -> Result<Proof> {

    let decoded_proof = sign::verify(ciphertest, theirpk);
    if decoded_proof.is_err() {
        return  Err(eyre!("Failed to verify proof"));
    }
    let proof = serde_json::from_slice(&decoded_proof.unwrap()).wrap_err_with(|| format!("Failed to parse proof"))?;

    Ok(proof)
}