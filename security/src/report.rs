use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::{self, PublicKey, SecretKey};
use sodiumoxide::crypto::aead::{self, Key};
use color_eyre::eyre::{Context, Result};

#[derive(Debug,Serialize,Deserialize)]
pub struct Report {
    epoch : usize,
    loc : (usize,usize),
    proofs : Vec<Vec<u8>>,
}

impl Report {
    pub fn new(epoch : usize, loc : (usize, usize), proofs : Vec<Vec<u8>>) -> Report {
        Report {
            epoch,
            loc,
            proofs,
        }
    }
}


pub fn encode_report(oursk : &SecretKey, theirpk : &PublicKey, report : Report) -> Vec<u8>{

    let plaintext = serde_json::to_vec(&report).unwrap();
    let nonce = box_::gen_nonce();

    let key_ = aead::gen_key();
    let aead_nonce = aead::gen_nonce();
    let box_nonce = box_::gen_nonce();

    //box_::seal(plaintext, &nonce, theirpk, oursk)
    aead::seal(&plaintext, None, &aead_nonce, &key_)
}

pub fn decode_report(oursk : &SecretKey, theirpk : &PublicKey, ciphertext : &Vec<u8>) -> Result<Report> {

    let nonce = box_::gen_nonce(); // TODO: Might need to be the same
    let decoded_report = box_::open(ciphertext, &nonce, theirpk, oursk).unwrap(); //TODO: fix this
    let report = serde_json::from_slice(&decoded_report)?;

    Ok(report)
}