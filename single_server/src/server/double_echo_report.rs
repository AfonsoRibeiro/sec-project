use std::sync::Arc;

use eyre::eyre;
use color_eyre::eyre::Result;
use sodiumoxide::crypto::{box_, sign};
use tonic::{Request, Response, Status, transport::Uri};
use security::{double_echo::{self, Write, success_echo}, key_management::{ServerKeys, ServerPublicKey}};
use protos::double_echo_broadcast::{EchoWriteRequest, EchoWriteResponse, double_echo_broadcast_client::DoubleEchoBroadcastClient, double_echo_broadcast_server::{DoubleEchoBroadcast}};

pub struct DoubleEcho{
    server_urls : Arc<Vec<Uri>>,
    necessary_res : usize, 
    f_servers : usize,
    server_keys : Arc<ServerKeys>,
    server_pkeys : Arc<ServerPublicKey>,
}

impl DoubleEcho {
    pub fn new(
        server_urls : Arc<Vec<Uri>>,
        necessary_res : usize, 
        f_servers : usize,
        server_keys : Arc<ServerKeys>,
        server_pkeys : Arc<ServerPublicKey>,
) -> DoubleEcho {

        DoubleEcho {
            server_urls,
            necessary_res,
            f_servers,
            server_keys,
            server_pkeys,
        }
    }

    pub fn confirm_write(
        message : Vec<u8>,
        client_id : usize,
    ){}
} 

/* 
CLIENT 
*/

pub async fn echo(
    url : &Uri,
    idx : usize,
    write : &double_echo::Write,
    sign_key : &sign::SecretKey,
    server_key : &box_::PublicKey,
) -> Result<()> {
    let (info, write, key) = double_echo::encode_echo_request(sign_key, server_key, write, idx);
    let mut client = DoubleEchoBroadcastClient::connect(url.clone()).await?;

    let request =
        tonic::Request::new( EchoWriteRequest{
            write,
            info,
        });

    match client.echo_write(request).await {
        Ok(response) => {
            let response = response.get_ref();
            if success_echo(&key, &response.nonce, &response.ok) {
                Ok(())
            } else {
                Err(eyre!("echo_write unable to validate server response"))
            }
        }
        Err(status) => { 
            println!("Echo write failed with code {:?} and message {:?}.",
            status.code(), status.message());
            Err(eyre!("Echo write failed with code {:?} and message {:?}.",
                            status.code(), status.message()))
        }
    }
}

/* 
SERVER 
*/
pub struct MyDoubleEchoWrite {
    echo : Arc<DoubleEcho>,
}

impl MyDoubleEchoWrite {
    pub fn new(
        echo : Arc<DoubleEcho>,
    ) -> MyDoubleEchoWrite {

        MyDoubleEchoWrite {
            echo,
        }
    }
} 

#[tonic::async_trait]
impl DoubleEchoBroadcast for MyDoubleEchoWrite { 
    async fn echo_write(
        &self,
        request : Request<EchoWriteRequest>,
    ) ->  Result<Response<EchoWriteResponse>, Status> {
        Err(Status::cancelled("Ola noffy!"))
    }
}
