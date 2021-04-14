use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::sealedbox;
use color_eyre::eyre::Result;
use crate::report::ReportInfo;
use eyre::eyre;

#[derive(Debug,Serialize,Deserialize)]
pub struct LocationReportRequest {
    idx : usize,
    epoch : usize,
}

impl LocationReportRequest {
    pub fn new(idx : usize, epoch : usize) -> LocationReportRequest {
        LocationReportRequest {
            idx,
            epoch,
        }
    }

    pub fn epoch(&self) -> usize { self.epoch }
    pub fn idx(&self) -> usize { self.idx }
}


#[derive(Debug,Serialize,Deserialize)]
pub struct LocationReportResponse {
    pub pos : (usize,usize)
}

impl LocationReportResponse {
    pub fn new(pos_x : usize, pos_y : usize) -> LocationReportResponse {
        LocationReportResponse {
            pos : (pos_x, pos_y),
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct UsersAtLocationRequest {
    pos : (usize,usize),
    epoch : usize,
}

impl UsersAtLocationRequest {
    pub fn new(pos : (usize, usize), epoch : usize) -> UsersAtLocationRequest {
        UsersAtLocationRequest {
            pos,
            epoch,
        }
    }

    pub fn pos(&self) -> (usize, usize) { self.pos }
    pub fn epoch(&self) -> usize { self.epoch }
}


#[derive(Debug,Serialize,Deserialize)]
pub struct UsersAtLocationResponse {
    pub idxs : Vec<usize>
}

impl UsersAtLocationResponse {
    pub fn new(idxs : Vec<usize>) -> UsersAtLocationResponse {
        UsersAtLocationResponse {
            idxs
        }
    }
}

pub fn encode_location_report(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    loc_report : &LocationReportRequest,
    idx : usize
) -> (Vec<u8>, Vec<u8>, secretbox::Key) {

    let plaintext = serde_json::to_vec(loc_report).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_report = secretbox::seal(&signtext,&box_nonce, &key);

    let info = ReportInfo::new(idx, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    (sealedbox::seal(&textinfo, theirpk), enc_report, key)
}

pub fn encoded_users_at_location_report(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    users_at_loc : &UsersAtLocationRequest,
    idx : usize
) -> (Vec<u8>, Vec<u8>, secretbox::Key) {

    let plaintext = serde_json::to_vec(users_at_loc).unwrap();
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

    let decoded_info = sealedbox::open(cipherinfo, ourpk, oursk).unwrap(); //TODO: fix unwrap
    let info = serde_json::from_slice(&decoded_info)?;

    Ok(info)
}

pub fn decode_loc_report(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<LocationReportRequest> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).unwrap(); //TODO: fix unwrap

    let report = sign::verify(&decoded_report,signpk).unwrap(); //TODO: fix unwrap

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}

pub fn decode_users_at_loc_report(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<UsersAtLocationRequest> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).unwrap(); //TODO: fix unwrap

    let report = sign::verify(&decoded_report,signpk).unwrap(); //TODO: fix unwrap

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}

pub fn decode_response_location(
    key : &secretbox::Key,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> Result<LocationReportResponse> {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        let decoded_response = secretbox::open(cyphertext, &nonce, key).unwrap(); //TODO: fix unwrap
        let response = serde_json::from_slice(&decoded_response)?;
        Ok(response)
    } else {
        Err(eyre!("Decode of location response failed."))
    }
}

pub fn decode_response_users_at_location(
    key : &secretbox::Key,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> Result<UsersAtLocationResponse> {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        let decoded_response = secretbox::open(cyphertext, &nonce, key).unwrap(); //TODO: fix unwrap
        let response = serde_json::from_slice(&decoded_response)?;
        Ok(response)
    } else {
        Err(eyre!("Decode of users at location response failed."))
    }
}