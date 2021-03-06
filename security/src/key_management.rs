use std::{collections::HashMap, io::{Read, Write}};
use std::fs;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use color_eyre::eyre::{Context, Result};
use eyre::eyre;

use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sign;
use sodiumoxide::crypto::secretbox;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientKeys {
    sign_key : sign::SecretKey,
    public_key : sign::PublicKey
}

impl ClientKeys {
    fn new(sign_key : sign::SecretKey, public_key : sign::PublicKey) -> ClientKeys {
        ClientKeys {
            sign_key,
            public_key,
        }
    }

    #[allow(dead_code)]
    pub fn sign_key(&self) -> &sign::SecretKey { &self.sign_key }

    #[allow(dead_code)]
    pub fn public_key(&self) -> &sign::PublicKey { &self.public_key }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HAClientKeys {
    private_key : sign::SecretKey,
    client_keys : HashMap<usize, sign::PublicKey>,
}

impl HAClientKeys {
    fn new(private_key : sign::SecretKey, client_keys : HashMap<usize, sign::PublicKey>,) -> HAClientKeys {
        HAClientKeys {
            private_key,
            client_keys,
        }
    }

    #[allow(dead_code)]
    pub fn sign_key(&self) -> &sign::SecretKey { &self.private_key }
    #[allow(dead_code)]
    pub fn client_public_key(&self, idx : usize) -> Option<&sign::PublicKey> {
        if let Some(sign) = self.client_keys.get(&idx) {
            Some(sign)
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub fn clients_public_keys(&self) -> &HashMap<usize, sign::PublicKey> {
        &self.client_keys
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPublicKey {
    public_keys : Vec<box_::PublicKey>,
    pub_sign_keys : Vec<sign::PublicKey>,
}

impl ServerPublicKey {
    fn new(public_keys : Vec<box_::PublicKey>, pub_sign_keys : Vec<sign::PublicKey>,) -> ServerPublicKey {
        ServerPublicKey {
            public_keys,
            pub_sign_keys,
        }
    }
    #[allow(dead_code)]
    pub fn public_key(&self, server_id : usize) -> &box_::PublicKey { &self.public_keys[server_id] }
    #[allow(dead_code)]
    pub fn public_keys(&self) -> &Vec<box_::PublicKey> { &self.public_keys }
    #[allow(dead_code)]
    pub fn public_sign_key(&self, server_id : usize) -> &sign::PublicKey { &self.pub_sign_keys[server_id] }
    #[allow(dead_code)]
    pub fn public_sign_keys(&self) -> &Vec<sign::PublicKey> { &self.pub_sign_keys }
}

#[derive(Debug, Serialize, Deserialize)]
struct Pass {
    pass : secretbox::Key,
    nonce : secretbox::Nonce,
}

impl Pass {
    fn new(pass : secretbox::Key, nonce : secretbox::Nonce) -> Pass {
        Pass {
            pass,
            nonce,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerKeys {
    private_key : box_::SecretKey,
    sign_key : sign::SecretKey,
    public_key : box_::PublicKey,
    client_keys : HashMap<usize, sign::PublicKey>,
    ha_public_key : sign::PublicKey,
}

impl ServerKeys{
    fn new(
        client_keys : HashMap<usize, sign::PublicKey>,
        private_key : box_::SecretKey,
        sign_key : sign::SecretKey,
        public_key : box_::PublicKey,
        ha_key : sign::PublicKey
    ) -> ServerKeys {
        ServerKeys {
            private_key,
            sign_key,
            public_key,
            client_keys,
            ha_public_key : ha_key,
        }
    }

    #[allow(dead_code)]
    pub fn private_key(&self) -> &box_::SecretKey { &self.private_key }
    #[allow(dead_code)]
    pub fn sign_key(&self) -> &sign::SecretKey { &self.sign_key }
    #[allow(dead_code)]
    pub fn public_key(&self) -> &box_::PublicKey { &self.public_key }
    #[allow(dead_code)]
    pub fn ha_public_key(&self) -> &sign::PublicKey { &self.ha_public_key }

    #[allow(dead_code)]
    pub fn client_sign_key(&self, idx : usize) -> Option<&sign::PublicKey> {
        self.client_keys.get(&idx)
    }
}

pub fn save_keys(n_clients : usize, n_servers : usize, keys_dir : String) -> Result<()> {
    let pass_dir = format!("{:}/pass", keys_dir);
    fs::create_dir_all(&keys_dir)?;
    fs::create_dir_all(&pass_dir)?;

    let mut clients_public_keys = HashMap::new();
    let mut client_secret_pairs =  HashMap::new();
    let mut servers_public_keys = vec![];
    let mut servers_pub_sign_keys = vec![];

    for index in 0..n_clients { //each index correspond to the idx of client
        let (signpk, signsk) = sign::gen_keypair();

        clients_public_keys.insert(index, signpk);

        let ck = ClientKeys::new(signsk, signpk);
        client_secret_pairs.insert(index, ck);
    }
    for (idx, c_k) in client_secret_pairs.into_iter() {
        save_client_keys(&keys_dir, idx, c_k)?;
    }

    let (ha_pk, ha_sk) = sign::gen_keypair();

    for server_idx in 0..n_servers {

        let (serverpk, serversk) = box_::gen_keypair();
        let (server_pub_sign, server_sign) = sign::gen_keypair();

        servers_public_keys.push(serverpk);
        servers_pub_sign_keys.push(server_pub_sign);

        save_server_keys(
            &keys_dir,
            server_idx,
            ServerKeys::new(
                clients_public_keys.clone(),
                serversk,
                server_sign,
                servers_public_keys[server_idx],
                ha_pk)
            )?;
    }


    save_ha_client_keys(&keys_dir, HAClientKeys::new(ha_sk, clients_public_keys))?;
    save_servers_public_keys(&keys_dir, ServerPublicKey::new(servers_public_keys, servers_pub_sign_keys))?;

    Ok(())

}

fn save_client_keys(keys_dir : &str, idx : usize, client : ClientKeys) -> Result<()> {
    let file = File::create(format!("{:}/client_{:04}.keys", keys_dir, idx))?;
    let pass_file = File::create(format!("{:}/pass/client_{:04}.keys", keys_dir, idx))?;

    let pass = Pass::new(
        secretbox::gen_key(),
        secretbox::gen_nonce(),
    );

    serde_json::to_writer(BufWriter::new(pass_file), &pass)?;

    let text = serde_json::to_vec(&client).unwrap();
    let encoded_keys = secretbox::seal(&text, &pass.nonce, &pass.pass);

    BufWriter::new(file).write_all(&encoded_keys)?;

    Ok(())
}

fn save_server_keys(keys_dir : &str, server_idx : usize, server : ServerKeys)  -> Result<()> {
    let file = File::create(format!("{:}/server_{:02}.keys", keys_dir, server_idx))?;
    let pass_file = File::create(format!("{:}/pass/server_{:02}.keys", keys_dir, server_idx))?;

    let pass = Pass::new(
        secretbox::gen_key(),
        secretbox::gen_nonce(),
    );

    serde_json::to_writer(BufWriter::new(pass_file), &pass)?;

    let text = serde_json::to_vec(&server).unwrap();
    let encoded_keys = secretbox::seal(&text, &pass.nonce, &pass.pass);

    BufWriter::new(file).write_all(&encoded_keys)?;

    Ok(())
}

fn save_servers_public_keys(keys_dir : &str, server_pub : ServerPublicKey) -> Result<()> {
    let file = File::create(format!("{:}/server_public.keys", keys_dir))?;

    serde_json::to_writer(BufWriter::new(file), &server_pub)?;

    Ok(())
}

fn save_ha_client_keys(keys_dir : &str, ha_keys : HAClientKeys) -> Result<()> {
    let file = File::create(format!("{:}/ha_client.keys", keys_dir))?;
    let pass_file = File::create(format!("{:}/pass/ha_client.keys", keys_dir))?;

    let pass = Pass::new(
        secretbox::gen_key(),
        secretbox::gen_nonce(),
    );

    serde_json::to_writer(BufWriter::new(pass_file), &pass)?;

    let text = serde_json::to_vec(&ha_keys).unwrap();
    let encoded_keys = secretbox::seal(&text, &pass.nonce, &pass.pass);

    BufWriter::new(file).write_all(&encoded_keys)?;

    Ok(())
}

#[allow(dead_code)]
pub fn retrieve_client_keys(keys_dir : &str, idx : usize) -> Result<ClientKeys> {
    let file = File::open(format!("{:}/client_{:04}.keys", keys_dir, idx))?;
    let pass_file = File::open(format!("{:}/pass/client_{:04}.keys", keys_dir, idx))?;

    let reader_pass = BufReader::new(pass_file);

    let pass : Pass = serde_json::from_reader(reader_pass)?;

    let mut reader = BufReader::new(file);

    let mut encoded_keys : Vec<u8> = vec![];
    reader.read_to_end(&mut encoded_keys)?;

    match secretbox::open(&encoded_keys, &pass.nonce, &pass.pass) {
        Ok(text) => Ok(serde_json::from_slice(&text).wrap_err_with(
                    || format!("Failed to parse struct ClientKeys from file '{:}'", format!("{:}/client_{:04}.keys", keys_dir, idx))
                )? ),
        Err(_) => Err(eyre!("retrieve_client_keys: unhable to decode keys")),
    }
}

#[allow(dead_code)]
pub fn retrieve_server_keys(keys_dir : &str, server_idx : usize)  -> Result<ServerKeys> {
    let file = File::open(format!("{:}/server_{:02}.keys", keys_dir, server_idx))?;
    let pass_file = File::open(format!("{:}/pass/server_{:02}.keys", keys_dir, server_idx))?;

    let reader_pass = BufReader::new(pass_file);

    let pass : Pass = serde_json::from_reader(reader_pass)?;

    let mut reader = BufReader::new(file);

    let mut encoded_keys : Vec<u8> = vec![];
    reader.read_to_end(&mut encoded_keys)?;

    match secretbox::open(&encoded_keys, &pass.nonce, &pass.pass) {
        Ok(text) => Ok(serde_json::from_slice(&text).wrap_err_with(
                    || format!("Failed to parse struct ServerKeys from file '{:}'", format!("{:}/server.keys", keys_dir))
                )? ),
        Err(_) => Err(eyre!("retrieve_client_keys: unhable to decode keys")),
    }

}

#[allow(dead_code)]
pub fn retrieve_servers_public_keys(keys_dir : &str) -> Result<ServerPublicKey> {
    let file = File::open(format!("{:}/server_public.keys", keys_dir))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct ServerPublicKey from file '{:}'", format!("{:}/server_public.keys", keys_dir))
    )? )
}

#[allow(dead_code)]
pub fn retrieve_ha_client_keys(keys_dir : &str) -> Result<HAClientKeys> {
    let file = File::open(format!("{:}/ha_client.keys", keys_dir))?;
    let pass_file = File::open(format!("{:}/pass/ha_client.keys", keys_dir))?;

    let reader_pass = BufReader::new(pass_file);

    let pass : Pass = serde_json::from_reader(reader_pass)?;

    let mut reader = BufReader::new(file);

    let mut encoded_keys : Vec<u8> = vec![];
    reader.read_to_end(&mut encoded_keys)?;

    match secretbox::open(&encoded_keys, &pass.nonce, &pass.pass) {
        Ok(text) => Ok(serde_json::from_slice(&text).wrap_err_with(
                    || format!("Failed to parse struct HAClientKeys from file '{:}'", format!("{:}/server_public.keys", keys_dir))
                )? ),
        Err(_) => Err(eyre!("retrieve_client_keys: unhable to decode keys")),
    }
}