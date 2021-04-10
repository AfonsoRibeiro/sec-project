use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sign;
use color_eyre::eyre::Result;

#[derive(Debug,Serialize,Deserialize)]
pub struct Report {
    epoch : usize,
    loc : (usize,usize),
    proofs : Vec<Vec<(usize ,u8)>>, //id + proofs
    idx : usize,
}

impl Report {
    pub fn new(epoch : usize, loc : (usize, usize), idx : usize,  proofs : Vec<Vec<(usize, u8)>>) -> Report {
        Report {
            epoch,
            loc,
            idx,
            proofs,
        }
    }
}


pub fn encode_report(signsk : &sign::SecretKey, oursk : &box_::SecretKey, theirpk : &box_::PublicKey, report : Report) -> Vec<u8>{

    let plaintext = serde_json::to_vec(&report).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let box_nonce = box_::gen_nonce();

    box_::seal(&signtext,&box_nonce, theirpk, oursk)
}

pub fn decode_report(signpk : &sign::PublicKey, oursk : &box_::SecretKey, theirpk : &box_::PublicKey, ciphertext : &Vec<u8>) -> Result<Report> {

    let nonce = box_::gen_nonce(); // TODO: Might need to be the same
    let decoded_report = box_::open(ciphertext, &nonce, theirpk, oursk).unwrap(); //TODO: fix this

    let report = sign::verify(&decoded_report,signpk).unwrap(); //TODO: fix this

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}