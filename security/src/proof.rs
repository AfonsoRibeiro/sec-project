use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign::{self, PublicKey, SecretKey};
use color_eyre::eyre::{Result, Context};
use eyre::eyre;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

    let decoded_proof = sign::verify(ciphertest, theirpk).map_err(|_| eyre!("Failed to verify proof"))?;

    let proof = serde_json::from_slice(&decoded_proof).wrap_err_with(|| format!("Failed to parse proof"))?;

    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPOCH : usize = 10;
    const IDX_REQ : usize = 5;
    const IDX_ASS : usize = 16;
    const LOC_ASS : (usize, usize) = (3, 6);

    #[test]
    fn create_proof() {
        let proof = Proof::new(EPOCH, IDX_REQ, IDX_ASS, LOC_ASS);
        assert_eq!(EPOCH, proof.epoch());
        assert_eq!(IDX_REQ, proof.idx_req());
        assert_eq!(IDX_ASS, proof.idx_ass());
        assert_eq!(LOC_ASS, proof.loc_ass());
    }

    #[test]
    fn sign_and_confirm_prood() {
        let proof = Proof::new(EPOCH, IDX_REQ, IDX_ASS, LOC_ASS);
        let proof_copy = Proof::new(EPOCH, IDX_REQ, IDX_ASS, LOC_ASS);

        let (pk, sk) = sign::gen_keypair();

        let signed_proof = sign_proof(&sk, proof);

        let verified_proof = verify_proof(&pk, &signed_proof);

        assert_eq!(proof_copy, verified_proof.unwrap())
    }

    #[test]
    fn sign_and_fail_check_proof() {
        let proof = Proof::new(EPOCH, IDX_REQ, IDX_ASS, LOC_ASS);

        let (_, sk) = sign::gen_keypair();
        let (bad_pk, _) = sign::gen_keypair();

        let signed_proof = sign_proof(&sk, proof);

        let verified_proof = verify_proof(&bad_pk, &signed_proof);

        assert!(verified_proof.is_err())

    }
}