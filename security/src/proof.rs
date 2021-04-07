use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::aead;

#[derive(Debug,Serialize,Deserialize)]
pub struct Proof {
    idx_req : usize,
    epoch : usize,
    idx_ass : usize,
    loc_ass : (usize,usize),
}

impl Proof {
    
    fn new(epoch : usize, idx_req : usize, idx_ass : usize, loc_ass : (usize, usize)) -> Proof {
        Proof {
            idx_req,
            epoch,
            idx_ass,
            loc_ass,
        }
    }
}


pub fn encode_proof(epoch : usize, idx_req : usize, idx_ass : usize, loc_ass : (usize, usize)) -> Vec<u8>{
    let proof = Proof::new(epoch, idx_req, idx_ass, loc_ass);
    let message = serde_json::to_string(&proof).unwrap();
    let k = aead::gen_key();
    let n = aead::gen_nonce();

    aead::seal(message.as_bytes(), None, &n, &k)
}

pub fn decode_proof(encoded_proof : Vec<u8>) {
    //aead::open(&encoded_proof, None, &n, &k).unwrap();
}