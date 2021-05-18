use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::sealedbox;
use color_eyre::eyre::Result;
use eyre::eyre;

#[derive(Debug, Serialize, Deserialize)]
pub struct Write{
    pub report : Vec<u8>,
    pub client_id : usize,
    pub epoch : usize,
    echo : bool,
}

impl Write {
    pub fn new_echo(report : Vec<u8>, client_id : usize, epoch : usize) -> Write{
        Write{
            report,
            client_id,
            epoch,
            echo : true,
        }
    }

    pub fn new_ready(report : Vec<u8>, client_id : usize, epoch : usize) -> Write{
        Write{
            report,
            client_id,
            epoch,
            echo : false,
        }
    }

    pub fn is_echo(&self) -> bool {
        self.echo
    }

    pub fn is_ready(&self) -> bool {
        !self.echo
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoInfo{
    pub server_id: usize,
    pub key: secretbox::Key,
    pub nonce: secretbox::Nonce,
}

impl EchoInfo{
    pub fn new(server_id : usize, key: secretbox::Key, nonce : secretbox::Nonce) -> EchoInfo{
        EchoInfo{
            server_id,
            key,
            nonce,
        }
    }
}

pub fn encode_echo_request(
    signsk : &sign::SecretKey,
    theirpk : &box_::PublicKey,
    write : &Write,
    server_id : usize,
) -> (Vec<u8>, Vec<u8>, secretbox::Key) {

    let plaintext = serde_json::to_vec(write).unwrap();
    let signtext = sign::sign(&plaintext, signsk);

    let key = secretbox::gen_key();
    let box_nonce = secretbox::gen_nonce();

    let enc_write = secretbox::seal(&signtext,&box_nonce, &key);

    let info = EchoInfo::new(server_id, key.clone(), box_nonce);
    let textinfo = serde_json::to_vec(&info).unwrap();

    (sealedbox::seal(&textinfo, theirpk), enc_write, key)
}

pub fn decode_echo_request(
    signpk : &sign::PublicKey,
    sim_key : &secretbox::Key,
    cipher_write : &Vec<u8>,
    nonce : &secretbox::Nonce,
) -> Result<(Write, Vec<u8>)> {

    let signed_echo_request = secretbox::open(cipher_write, nonce, sim_key).map_err(|_| eyre!("decoded_echo_request: Unable to open secretbox"))?;
    let decoded_echo_request = sign::verify(&signed_echo_request, signpk).map_err(|_| eyre!("decoded_echo_request: Unable to verify signature"))?;

    let echo_request = serde_json::from_slice(&decoded_echo_request)?;

    Ok((echo_request, decoded_echo_request))
}

pub fn decode_echo_info(
    oursk : &box_::SecretKey,
    ourpk : &box_::PublicKey,
    cipherinfo : &Vec<u8>,
) -> Result<EchoInfo> {

    let decoded_echo_info = sealedbox::open(cipherinfo, ourpk, oursk).map_err(|_| eyre!("decode_echo_info: Unable to open sealedbox"))?;
    let info = serde_json::from_slice(&decoded_echo_info)?;

    Ok(info)
}

pub fn success_echo(
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
