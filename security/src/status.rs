use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::sealedbox;
use color_eyre::eyre::Result;
use crate::report::ReportInfo;
use eyre::eyre;


pub fn decode_info(
    oursk : &box_::SecretKey,
    ourpk : &box_::PublicKey,
    cipherinfo : &Vec<u8>,
) -> Result<ReportInfo> {

    let decoded_info = sealedbox::open(cipherinfo, ourpk, oursk).map_err(|_| eyre!("decode_info: Unable to open sealbox"))?;
    let info = serde_json::from_slice(&decoded_info)?;

    Ok(info)
}

/**
 * Obtain Location
 */

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

pub fn decode_loc_report(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<LocationReportRequest> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).map_err(|_| eyre!("decode_loc_report: Unable to open secretbox"))?;
    let report = sign::verify(&decoded_report,signpk).map_err(|_| eyre!("decode_loc_report: Unable to verify signature"))?;

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}

//

pub fn encode_loc_response(
    key : &secretbox::Key,
    x : usize,
    y : usize,
) -> (Vec<u8>, secretbox::Nonce) {

    let nonce = secretbox::gen_nonce();

    let loc = LocationReportResponse::new(x, y);
    let plaintext = serde_json::to_vec(&loc).unwrap();
    (secretbox::seal(&plaintext, &nonce, key), nonce)
}

pub fn decode_response_location(
    key : &secretbox::Key,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> Result<LocationReportResponse> {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        let decoded_response = secretbox::open(cyphertext, &nonce, key).map_err(|_| eyre!("decode_response_location: Unable to open secretbox"))?;
        let response = serde_json::from_slice(&decoded_response)?;
        Ok(response)
    } else {
        Err(eyre!("Decode of location response failed."))
    }
}
/**
 * Obtain Users at Location
 */

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

pub fn encode_users_at_location_report(
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

pub fn decode_users_at_loc_report(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<UsersAtLocationRequest> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).map_err(|_| eyre!("decode_users_at_loc_report: Unable to open secretbox"))?;
    let report = sign::verify(&decoded_report,signpk).map_err(|_| eyre!("decode_users_at_loc: Unable to verify signature"))?;

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}



pub fn encode_users_at_loc_response(
    key : &secretbox::Key,
    idxs : Vec<usize>,
) -> (Vec<u8>, secretbox::Nonce) {

    let nonce = secretbox::gen_nonce();

    let loc = UsersAtLocationResponse::new(idxs);
    let plaintext = serde_json::to_vec(&loc).unwrap();
    (secretbox::seal(&plaintext, &nonce, key), nonce)
}


pub fn decode_users_at_loc_response(
    key : &secretbox::Key,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> Result<UsersAtLocationResponse> {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        let decoded_response = secretbox::open(cyphertext, &nonce, key).map_err(|_| eyre!("decode_users_at_loc_response: Unable to open secretbox"))?;
        let response = serde_json::from_slice(&decoded_response)?;
        Ok(response)
    } else {
        Err(eyre!("Decode of users at location response failed."))
    }
}

/**
 * Request My Proofs
 */


#[derive(Debug,Serialize,Deserialize)]
pub struct MyProofsRequest {
    pub epochs : Vec<usize>,
}

impl MyProofsRequest {
    pub fn new(epochs : Vec<usize>) -> MyProofsRequest {
        MyProofsRequest {
            epochs,
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct MyProofsResponse {
    pub proofs : Vec<Vec<u8>>
}

impl MyProofsResponse {
    pub fn new(proofs : Vec<Vec<u8>>) -> MyProofsResponse {
        MyProofsResponse {
            proofs
        }
    }
}

pub fn encode_my_proofs_request(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    my_proofs : &MyProofsRequest,
    idx : usize
) -> (Vec<u8>, Vec<u8>, secretbox::Key) {

    let plaintext = serde_json::to_vec(my_proofs).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_epochs = secretbox::seal(&signtext,&box_nonce, &key);

    let info = ReportInfo::new(idx, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    (sealedbox::seal(&textinfo, theirpk), enc_epochs, key)
}

pub fn decode_my_proofs_report(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<MyProofsRequest> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).map_err(|_| eyre!("decode_my_proofs_report: Unable to open secretbox"))?;
    let report = sign::verify(&decoded_report,signpk).map_err(|_| eyre!("decode_my_proofs: Unable to verify signature"))?;

    let report = serde_json::from_slice(&report)?;

    Ok(report)
}

//

pub fn encode_my_proofs_response(
    key : &secretbox::Key,
    proofs : Vec<Vec<u8>>,
) -> (Vec<u8>, secretbox::Nonce) {

    let nonce = secretbox::gen_nonce();

    let loc = MyProofsResponse::new(proofs);
    let plaintext = serde_json::to_vec(&loc).unwrap();
    (secretbox::seal(&plaintext, &nonce, key), nonce)
}


pub fn decode_my_proofs_response(
    key : &secretbox::Key,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> Result<MyProofsResponse> {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        let decoded_response = secretbox::open(cyphertext, &nonce, key).map_err(|_| eyre!("decode_my_proofs_response: Unable to open secretbox"))?;
        let response = serde_json::from_slice(&decoded_response)?;
        Ok(response)
    } else {
        Err(eyre!("Decode of users at location response failed."))
    }
}
