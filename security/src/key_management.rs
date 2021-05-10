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
    pub fn client_sign_key(&self, idx : usize) -> Option<&sign::PublicKey> {
        if let Some(sign) = self.client_keys.get(&idx) {
            Some(sign)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPublicKey {
    public_keys : Vec<box_::PublicKey>,
}

impl ServerPublicKey {
    fn new(public_keys : Vec<box_::PublicKey>) -> ServerPublicKey {
        ServerPublicKey {
            public_keys,
        }
    }
    #[allow(dead_code)]
    pub fn public_key(&self, server_id : usize) -> &box_::PublicKey { &self.public_keys[server_id] }
    #[allow(dead_code)]
    pub fn public_keys(&self) -> &Vec<box_::PublicKey> { &self.public_keys }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerKeys {
    private_key : box_::SecretKey,
    public_key : box_::PublicKey,
    client_keys : HashMap<usize, sign::PublicKey>,
    ha_public_key : sign::PublicKey,
}

impl ServerKeys{
    fn new(
        client_keys : HashMap<usize, sign::PublicKey>,
        private_key : box_::SecretKey,
        public_key : box_::PublicKey,
        ha_key : sign::PublicKey
    ) -> ServerKeys {
        ServerKeys {
            private_key,
            public_key,
            client_keys,
            ha_public_key : ha_key,
        }
    }

    #[allow(dead_code)]
    pub fn private_key(&self) -> &box_::SecretKey { &self.private_key }
    #[allow(dead_code)]
    pub fn public_key(&self) -> &box_::PublicKey { &self.public_key }
    #[allow(dead_code)]
    pub fn ha_public_key(&self) -> &sign::PublicKey { &self.ha_public_key }

    #[allow(dead_code)]
    pub fn client_sign_key(&self, idx : usize) -> Option<&sign::PublicKey> {
        if let Some(sign) = self.client_keys.get(&idx) {
            Some(sign)
        } else {
            None
        }
    }
}

pub fn save_keys(n_clients : usize, n_servers : usize, keys_dir : String) -> Result<()> {
    fs::create_dir_all(&keys_dir)?;

    let mut clients_public_keys = HashMap::new();
    let mut client_secret_pairs =  HashMap::new();
    let mut servers_public_keys = vec![];

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

        let (serverpk, serversk)= box_::gen_keypair();

        servers_public_keys.push(serverpk);

        save_server_keys(
            &keys_dir,
            server_idx,
            ServerKeys::new(
                clients_public_keys.clone(),
                serversk,
                servers_public_keys[server_idx],
                ha_pk)
            )?;
    }


    save_ha_client_keys(&keys_dir, HAClientKeys::new(ha_sk, clients_public_keys))?;
    save_servers_public_keys(&keys_dir, ServerPublicKey::new(servers_public_keys))?;

    Ok(())

}

fn save_client_keys(keys_dir : &str, idx : usize, client : ClientKeys) -> Result<()> {
    let file = File::create(format!("{:}/client_{:04}.keys", keys_dir, idx))?;

    serde_json::to_writer(BufWriter::new(file), &client)?;

    Ok(())
}

fn save_server_keys(keys_dir : &str, server_idx : usize, server : ServerKeys)  -> Result<()> {
    let file = File::create(format!("{:}/server_{:02}.keys", keys_dir, server_idx))?;

    serde_json::to_writer(BufWriter::new(file), &server)?;

    Ok(())
}

fn save_servers_public_keys(keys_dir : &str, server_pub : ServerPublicKey) -> Result<()> {
    let file = File::create(format!("{:}/server_public.keys", keys_dir))?;

    serde_json::to_writer(BufWriter::new(file), &server_pub)?;

    Ok(())
}

fn save_ha_client_keys(keys_dir : &str, ha_keys : HAClientKeys) -> Result<()> {
    let file = File::create(format!("{:}/ha_client.keys", keys_dir))?;

    serde_json::to_writer(BufWriter::new(file), &ha_keys)?;

    Ok(())
}

#[allow(dead_code)]
pub fn retrieve_client_keys(keys_dir : &str, idx : usize) -> Result<ClientKeys> {
    let file = File::open(format!("{:}/client_{:04}.keys", keys_dir, idx))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct ClientKeys from file '{:}'", format!("{:}/client_{:04}.keys", keys_dir, idx))
    )? )
}

#[allow(dead_code)]
pub fn retrieve_server_keys(keys_dir : &str, server_idx : usize)  -> Result<ServerKeys> {
    let file = File::open(format!("{:}/server_{:02}.keys", keys_dir, server_idx))?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct ServerKeys from file '{:}'", format!("{:}/server.keys", keys_dir))
    )? )
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
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader).wrap_err_with(
        || format!("Failed to parse struct HAClientKeys from file '{:}'", format!("{:}/server_public.keys", keys_dir))
    )? )
}