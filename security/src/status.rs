use std::collections::HashSet;

use pow::Pow;
use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::sealedbox;
use color_eyre::eyre::Result;
use crate::{DIFICULTY, proof::{self, Proof}, report::ReportInfo};
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
    pub report : Vec<u8>,
}

impl LocationReportResponse {
    pub fn new(report : Vec<u8>) -> LocationReportResponse {
        LocationReportResponse {
            report,
        }
    }
}

pub fn encode_location_report(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    loc_report : &LocationReportRequest,
    idx : usize
) -> (Vec<u8>, Vec<u8>, secretbox::Key, Vec<u8>) {

    let plaintext = serde_json::to_vec(loc_report).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_report = secretbox::seal(&signtext,&box_nonce, &key);

    let info = ReportInfo::new(idx, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    let encoded_textinfo =sealedbox::seal(&textinfo, theirpk);

    let pw = Pow::prove_work(&encoded_textinfo, DIFICULTY).unwrap();
    let vec_pw  = serde_json::to_vec(&pw).unwrap();

    (encoded_textinfo, enc_report, key, vec_pw)
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
    report : Vec<u8>,
) -> (Vec<u8>, secretbox::Nonce) {

    let nonce = secretbox::gen_nonce();

    let loc = LocationReportResponse::new(report);
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
    pub idxs_reports : Vec<(usize, Vec<u8>)>,
}

impl UsersAtLocationResponse {
    pub fn new(idxs_reports : Vec<(usize, Vec<u8>)>) -> UsersAtLocationResponse {
        UsersAtLocationResponse {
            idxs_reports ,
        }
    }
}

pub fn encode_users_at_location_report(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    users_at_loc : &UsersAtLocationRequest,
    idx : usize
) -> (Vec<u8>, Vec<u8>, secretbox::Key, Vec<u8>) {

    let plaintext = serde_json::to_vec(users_at_loc).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_report = secretbox::seal(&signtext,&box_nonce, &key);

    let info = ReportInfo::new(idx, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    let encoded_textinfo = sealedbox::seal(&textinfo, theirpk);

    let pw = Pow::prove_work(&encoded_textinfo, DIFICULTY).unwrap();
    let vec_pw  = serde_json::to_vec(&pw).unwrap();

    (encoded_textinfo, enc_report, key, vec_pw)
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
    idxs_reports : Vec<(usize, Vec<u8>)>,
) -> (Vec<u8>, secretbox::Nonce) {

    let nonce = secretbox::gen_nonce();

    let loc = UsersAtLocationResponse::new(idxs_reports);
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
    pub epochs : HashSet<usize>,
}

impl MyProofsRequest {
    pub fn new(epochs : HashSet<usize>) -> MyProofsRequest {
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
) -> (Vec<u8>, Vec<u8>, secretbox::Key, Vec<u8>) {

    let plaintext = serde_json::to_vec(my_proofs).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_epochs = secretbox::seal(&signtext,&box_nonce, &key);

    let info = ReportInfo::new(idx, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    let encoded_textinfo = sealedbox::seal(&textinfo, theirpk);

    let pw = Pow::prove_work(&encoded_textinfo, DIFICULTY).unwrap();
    let vec_pw  = serde_json::to_vec(&pw).unwrap();

    (encoded_textinfo, enc_epochs, key, vec_pw)
}

pub fn decode_my_proofs_request(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipherreport : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<MyProofsRequest> {

    let decoded_report = secretbox::open(cipherreport, nonce, sim_key).map_err(|_| eyre!("decode_my_proofs_request: Unable to open secretbox"))?;
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
    public_key : &sign::PublicKey,
    nonce :&Vec<u8>,
    cyphertext : &Vec<u8>,
) -> Result<Vec<Proof>> {
    if let Some(nonce) = secretbox::Nonce::from_slice(nonce) {
        let decoded_response = secretbox::open(cyphertext, &nonce, key).map_err(|_| eyre!("decode_my_proofs_response: Unable to open secretbox"))?;
        let response : MyProofsResponse = serde_json::from_slice(&decoded_response)?;
        let mut result = vec![];

        for proof in response.proofs.iter() {
            if let Ok(x) = proof::verify_proof(public_key, proof) {
                result.push(x);
            }
            else {
                return Err(eyre!("Decode of users at location response failed."))
            }
        }
        Ok(result)
    } else {
        Err(eyre!("Decode of users at location response failed."))
    }
}
