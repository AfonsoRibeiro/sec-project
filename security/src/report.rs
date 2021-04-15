use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::sealedbox;
use color_eyre::eyre::Result;
use eyre::eyre;

#[derive(Debug,Serialize,Deserialize)]
pub struct Report {
    epoch : usize,
    loc : (usize,usize),
    proofs : Vec<(usize ,Vec<u8>)>, //id + proofs
    idx : usize,
}

impl Report {
    pub fn new(epoch : usize, loc : (usize, usize), idx : usize, idxs_ass : Vec<usize>, proofs : Vec<Vec<u8>>) -> Report {
        let report = idxs_ass.into_iter().zip(proofs).collect();
        Report {
            epoch,
            loc,
            idx,
            proofs : report,
        }
    }

    pub fn epoch(&self) -> usize { self.epoch }
    pub fn idx(&self) -> usize { self.idx }
    pub fn loc(&self) -> (usize, usize) { self.loc }
    pub fn proofs(&self) -> &Vec<(usize ,Vec<u8>)> { &self.proofs }
}


#[derive(Debug,Serialize,Deserialize)]
pub struct ReportInfo {
    idx : usize,
    key : secretbox::Key,
    nonce : secretbox::Nonce,
}

impl ReportInfo {
    pub fn new(idx : usize, key : secretbox::Key, nonce : secretbox::Nonce) -> ReportInfo {
        ReportInfo {
            idx,
            key,
            nonce,
        }
    }

    pub fn idx(&self) -> usize { self.idx }
    pub fn key(&self) -> &secretbox::Key { &self.key }
    pub fn nonce(&self) -> &secretbox::Nonce { &self.nonce }
}


pub fn encode_report(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    report : &Report,
    idx : usize
) -> (Vec<u8>, Vec<u8>, secretbox::Key) {

    let plaintext = serde_json::to_vec(report).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_report = secretbox::seal(&signtext,&box_nonce, &key);

    let info = ReportInfo::new(idx, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    (sealedbox::seal(&textinfo, theirpk), enc_report, key)
}

pub fn decode_info(
    oursk : &box_::SecretKey,
    ourpk : &box_::PublicKey,
    cipherinfo : &Vec<u8>,
) -> Result<ReportInfo> {

    let decoded_info = sealedbox::open(cipherinfo, ourpk, oursk).map_err(|_| eyre!("decode_info: Unable to open sealedbox"))?; 
    let info = serde_json::from_slice(&decoded_info)?;

    Ok(info)
}

pub fn decode_report(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<Report> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).map_err(|_| eyre!("decoded_report: Unable to open secretbox"))?; 
    let report = sign::verify(&decoded_report,signpk).map_err(|_| eyre!("decoded_report: Unable to verify report"))?; 

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}

pub fn success_report(
    key : &secretbox::Key,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> bool {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        secretbox::open(cyphertext, &nonce, key).is_ok()
    } else {
        false
    }
}