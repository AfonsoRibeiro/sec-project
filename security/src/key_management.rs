use std::collections::HashMap;
use std::fs;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use color_eyre::eyre::{Context, Result};

use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sign;


#[derive(Debug, Serialize, Deserialize)]
pub struct ClientKeys {
    sign_key : sign::SecretKey,
    private_key : box_::SecretKey,
}

impl ClientKeys {
    fn new(sign_key : sign::SecretKey, private_key : box_::SecretKey,) -> ClientKeys {
        ClientKeys {
            sign_key,
            private_key,
        }
    }

    pub fn sign_key(&self) -> sign::SecretKey { self.sign_key.clone() }
    pub fn private_key(&self) -> box_::SecretKey { self.private_key.clone() }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HAClientKeys {
    private_key : box_::SecretKey,
}

impl HAClientKeys {
    fn new(private_key : box_::SecretKey,) -> HAClientKeys {
        HAClientKeys {
            private_key,
        }
    }

    pub fn private_key(&self) -> box_::SecretKey { self.private_key.clone() }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPublicKey {
    public_key : box_::PublicKey,
}

impl ServerPublicKey {
    fn new(public_key : box_::PublicKey) -> ServerPublicKey {
        ServerPublicKey {
            public_key,
        }
    }
    pub fn public_key(&self) -> box_::PublicKey { self.public_key.clone() }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerKeys {
    private_key : box_::SecretKey,
    public_keys : HashMap<usize, (box_::PublicKey, sign::PublicKey)>,
    ha_public_key : box_::PublicKey,
}

impl ServerKeys{
    fn new(public_keys : HashMap<usize, (box_::PublicKey, sign::PublicKey)>, private_key : box_::SecretKey, ha_key : box_::PublicKey) -> ServerKeys {
        ServerKeys {
            private_key,
            public_keys,
            ha_public_key : ha_key,
        }
    }

    pub fn private_key(&self) -> box_::SecretKey { self.private_key.clone() }
    pub fn ha_public_key(&self) -> box_::PublicKey { self.ha_public_key.clone() }
}

pub fn save_keys(size : usize, keys_dir : String) -> Result<()> {
    fs::create_dir_all(&keys_dir)?;

    let mut key_pairs = HashMap::new();
    let mut client_secret_pairs =  HashMap::new();

    let (serverpk, serversk)= box_::gen_keypair();

    for index in 0..size { //each index correspond to the idx of client

        let (ourpk, oursk) = box_::gen_keypair();
        let (pk, sk) = sign::gen_keypair();

        key_pairs.insert(index, (ourpk, pk));

        let ck = ClientKeys::new(sk, oursk);
        client_secret_pairs.insert(index, ck);
    }

    let (ha_pk, ha_sk) = box_::gen_keypair();

    let sk = ServerKeys::new(key_pairs, serversk, ha_pk);
    let server_public_key = ServerPublicKey::new(serverpk);

    for (idx, c_k) in client_secret_pairs.into_iter() {
        save_client_keys(&keys_dir, idx, c_k)?;
    }

    save_ha_client_keys(&keys_dir, HAClientKeys::new(ha_sk))?;
    save_server_keys(&keys_dir, sk)?;
    save_server_public_keys(&keys_dir, server_public_key)?;

    Ok(())

}

fn save_client_keys(keys_dir : &str, idx : usize, client : ClientKeys) -> Result<()> {
    let file = File::create(format!("{:}/client_{:04}.keys", keys_dir, idx))?;

    serde_json::to_writer(BufWriter::new(file), &client)?;

    Ok(())
}

fn save_server_keys(keys_dir : &str, server : ServerKeys)  -> Result<()> {
    let file = File::create(format!("{:}/server.keys", keys_dir))?;

    serde_json::to_writer(BufWriter::new(file), &server)?;

    Ok(())
}

fn save_server_public_keys(keys_dir : &str, server_pub : ServerPublicKey) -> Result<()> {
    let file = File::create(format!("{:}/server_public.keys", keys_dir))?;

    serde_json::to_writer(BufWriter::new(file), &server_pub)?;

    Ok(())
}

fn save_ha_client_keys(keys_dir : &str, ha_keys : HAClientKeys) -> Result<()> {
    let file = File::create(format!("{:}/ha_client.keys", keys_dir))?;

    serde_json::to_writer(BufWriter::new(file), &ha_keys)?;

    Ok(())
}

pub fn retrieve_client_keys(keys_dir : &str, idx : usize) -> Result<ClientKeys> {
    let file = File::create(format!("{:}/client_{:04}.keys", keys_dir, idx))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", format!("{:}/client_{:04}.keys", keys_dir, idx))
    )? )
}

pub fn retrieve_server_keys(keys_dir : &str)  -> Result<ServerKeys> {
    let file = File::create(format!("{:}/server.keys", keys_dir))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", format!("{:}/server.keys", keys_dir))
    )? )
}

pub fn retrieve_server_public_keys(keys_dir : &str) -> Result<ServerPublicKey> {
    let file = File::create(format!("{:}/server_public.keys", keys_dir))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", format!("{:}/server_public.keys", keys_dir))
    )? )
}

pub fn retrieve_ha_client_keys(keys_dir : &str) -> Result<HAClientKeys> {
    let file = File::create(format!("{:}/ha_client.keys", keys_dir))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct Timeline from file '{:}'", format!("{:}/server_public.keys", keys_dir))
    )? )
}