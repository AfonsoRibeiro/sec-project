
use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::sign;


#[derive(Debug,Serialize,Deserialize)]
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
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ServerPublicKey {
    public_key : box_::PublicKey,
}   

impl ServerPublicKey {
    fn new(public_key : sign::PublicKey) -> ServerPublicKey {
        ServerPublicKey {
            public_key,
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ServerKeys {
    private_key : box_::SecretKey,
    public_keys : HashMap<usize, (box_::PublicKey, sign::PublicKey)>,
}

impl ServerKeys{
    fn new(public_keys : HashMap<usize, (box_::PublicKey, sign::PublicKey)>, private_key : box_::SecretKey) -> ServerKeys {
        ServerKeys {
            private_key,
            public_keys
        }
    }
}

fn save_keys( size : usize) {
    let key_pairs = HashMap::new();
    let client_secret_pairs =  HashMap::new();

    let (serverpk, serversk)= box_::gen_keypair();
    
    for index in 0..size { //each index correspond to the idx of client
        
        let (ourpk, oursk)= box_::gen_keypair();
        let (pk, sk)= sign::gen_keypair();

        key_pairs.insert(index, (ourpk, pk));

        let ck = ClientKeys::new(sk, oursk);
        client_secret_pairs.insert(index, ck);
    }

    let sk = ServerKeys::new(key_pairs, serversk);
    let server_public_key = ServerPublicKey::new(serverpk);


    
}

fn save_client_keys(idx : usize, client : ClientKeys) {
    
}

// pub fn save_key_pair(file_name : &str, timeline : &Timeline) -> Result<()> {
//    let file = File::create(file_name)?;

//     serde_json::to_writer(BufWriter::new(file), timeline)?;

//     Ok(())
// }

// pub fn retrieve_key_pair(file_name : &str) -> Result<Timeline> {
//     let file = File::open(file_name)?;
//     let reader = BufReader::new(file);

//     Ok(serde_json::from_reader(reader).wrap_err_with(
//         || format!("Failed to parse struct Timeline from file '{:}'", file_name)
//     )? )
// }