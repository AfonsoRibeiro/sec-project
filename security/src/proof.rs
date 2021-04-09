use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign::{self, PublicKey, SecretKey};
use color_eyre::eyre::Result;

#[derive(Debug,Serialize,Deserialize)]
pub struct Proof {
    idx_req : usize,
    epoch : usize,
    loc_ass : (usize,usize),
}

impl Proof {
    pub fn new(epoch : usize, idx_req : usize, loc_ass : (usize, usize)) -> Proof {
        Proof {
            idx_req,
            epoch,
            loc_ass,
        }
    }
}

pub fn sign_proof(oursk : &SecretKey, proof : Proof) -> Vec<u8>{

    let plaintext = serde_json::to_vec(&proof).unwrap();

    sign::sign(&plaintext, oursk)
}

pub fn verify_proof(theirpk : &PublicKey, ciphertest : &Vec<u8>) -> Result<Proof> {

    let decoded_proof = sign::verify(ciphertest, theirpk).unwrap(); //TODO: fix this
    let proof = serde_json::from_slice(&decoded_proof)?;

    Ok(proof)
}