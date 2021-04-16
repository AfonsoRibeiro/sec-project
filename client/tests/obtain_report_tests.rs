mod common;

use client::reports;
use tokio::time::sleep;
use tonic::transport::Uri;

use std::time::Duration;


/*
    Assumes client 19 as been launched
*/

const IDX : usize = 19;
const NOT_MINE_IDX : usize = 17;
const EPOCH : usize = 0;
const N_EPOCHS : usize = 10;

#[tokio::test]
#[ignore]
pub async fn get_submited_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();

    sleep(Duration::from_millis(2000)).await; //allow time for user to have submited report

    if let Some(location) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let loc_res =
            reports::obtain_location_report(
                timeline,
                IDX,
                EPOCH,
                server_url,
                client_keys.sign_key(),
                sever_key
            ).await;

        assert!(loc_res.is_ok());
        assert_eq!(location, loc_res.unwrap());
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}


#[tokio::test]
#[ignore]
pub async fn get_not_mine_submitted_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();

    sleep(Duration::from_millis(2000)).await; //allow time for user to have submited report

    let loc_res =
        reports::obtain_location_report(
            timeline,
            NOT_MINE_IDX,
            EPOCH,
            server_url,
            client_keys.sign_key(),
            sever_key
        ).await;

    assert!(loc_res.is_err());
}

#[tokio::test]
#[ignore]
pub async fn get_not_submitted_yet_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();

    let loc_res =
        reports::obtain_location_report(
            timeline,
            NOT_MINE_IDX,
            N_EPOCHS,
            server_url,
            client_keys.sign_key(),
            sever_key
        ).await;

    assert!(loc_res.is_err());
}
