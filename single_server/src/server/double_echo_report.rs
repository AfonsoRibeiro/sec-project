use std::sync::Arc;

use eyre::eyre;
use color_eyre::eyre::Result;

use tonic::{Request, Response, Status, transport::Uri};

use protos::double_echo_broadcast::{EchoReportRequest, EchoReportResponse, ReadyReportRequest, ReadyReportResponse, double_echo_broadcast_client::DoubleEchoBroadcastClient, double_echo_broadcast_server::{DoubleEchoBroadcast}};

/* 
SERVER EM GRANDE 
*/
pub struct MyDoubleEchoReport {
    server_urls : Arc<Vec<Uri>>,
    necessary_res : usize, 
    f_servers : usize
}

impl MyDoubleEchoReport {
    pub fn new(
        server_urls : Arc<Vec<Uri>>,
        necessary_res : usize, 
        f_servers : usize
    ) -> MyDoubleEchoReport {

        MyDoubleEchoReport {
            server_urls,
            necessary_res,
            f_servers
        }
    }
} 

#[tonic::async_trait]
impl DoubleEchoBroadcast for MyDoubleEchoReport { 
    async fn echo_report(
        &self,
        request : Request<EchoReportRequest>,
    ) ->  Result<Response<EchoReportResponse>, Status> {
        Err(Status::cancelled("Ola noffy!"))
    }

    async fn ready_report (
        &self,
        request : Request<ReadyReportRequest>,
    ) ->  Result<Response<ReadyReportResponse>, Status> {
        Err(Status::cancelled("Ola noffy!"))
    }
}
/* 
CLIENT EM GRANDE 
*/

pub async fn echo(url : Uri) -> Result<()> {
    let mut client = DoubleEchoBroadcastClient::connect(url).await?;

    Ok(())
}

//Todo see if this is necessary
pub async fn ready(url : Uri) -> Result<()> {
    let mut client = DoubleEchoBroadcastClient::connect(url).await?;
    
    Ok(())
}
