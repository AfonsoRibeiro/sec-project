mod common;

use ha_client::verifying::{obtain_location_report, obtain_users_at_location};

use tokio::time::sleep;
use tonic::transport::Uri;

use std::time::Duration;


const IDX : usize = 10;
const N_IDS : usize = 20;
const INVALID_ID : usize = 50;
const EPOCH : usize = 0;
const POS_X : usize = 1;
const POS_Y : usize = 0;
const GRID_SIZE : usize = 3;
const N_EPOCHS : usize = 10;

#[tokio::test]
#[ignore]
pub async fn get_submited_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let ha_client_keys = common::get_ha_client_keys();
    let server_key = common::get_pub_server_key();

    let timeline = common::get_timeline();

    sleep(Duration::from_millis(2000)).await; //allow time for user to have submited report

    if let Some(location) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let loc_res =
            obtain_location_report(
                IDX,
                EPOCH,
                GRID_SIZE,
                server_url,
                &ha_client_keys.sign_key(),
                &server_key[0],
                ha_client_keys.client_public_key(0).unwrap(),
            ).await;

        assert!(loc_res.is_ok());
        assert_eq!(location, loc_res.unwrap());
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}

#[tokio::test]
#[ignore]
pub async fn get_not_submited_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let ha_client_keys = common::get_ha_client_keys();
    let server_key = common::get_pub_server_key();

    let loc_res =
        obtain_location_report(
            IDX,
            N_EPOCHS,
            GRID_SIZE,
            server_url,
            &ha_client_keys.sign_key(),
            &server_key[0],
            ha_client_keys.client_public_key(0).unwrap(),
        ).await;

    assert!(loc_res.is_err());
}

#[tokio::test]
#[ignore]
pub async fn get_invalid_id_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let ha_client_keys = common::get_ha_client_keys();
    let server_key = common::get_pub_server_key();

    let loc_res =
        obtain_location_report(
            INVALID_ID,
            EPOCH,
            GRID_SIZE,
            server_url,
            ha_client_keys.sign_key(),
            &server_key[0],
            ha_client_keys.client_public_key(0).unwrap(),
        ).await;

    assert!(loc_res.is_err());
}

#[tokio::test]
#[ignore]
pub async fn get_location_report_invalid_signature () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let server_key = common::get_pub_server_key();

    let loc_res =
        obtain_location_report(
            IDX,
            N_EPOCHS,
            GRID_SIZE,
            server_url,
            &client_keys.sign_key(),
            &server_key[0],
            client_keys.public_key(),
        ).await;

    assert!(loc_res.is_err());
}

#[tokio::test]
#[ignore]
pub async fn get_users_at_location_at_epoch () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let ha_client_keys = common::get_ha_client_keys();
    let server_key = common::get_pub_server_key();

    let timeline = common::get_timeline();

    sleep(Duration::from_millis(2000)).await; //allow time for user to have submited report

    let mut users = vec![];

    for idx in 0..N_IDS {
        if let Some((x,y)) = timeline.get_location_at_epoch(idx, EPOCH) {
            if POS_X == x && POS_Y == y {
                users.push(idx);
            }
        }
    }

    let users_res =
        obtain_users_at_location(
            EPOCH,
            POS_X,
            POS_Y,
            server_url,
            ha_client_keys.sign_key(),
            &server_key[0],
            ha_client_keys.clients_public_keys(),
        ).await;

    assert!(users_res.is_ok());

    let users_res = users_res.unwrap();

    assert_eq!(users.len(), users_res.len());

    for idx in users.into_iter() { // check if all ids match
        assert!(users_res.iter().any(|&id| id == idx));
    }
}

#[tokio::test]
#[ignore]
pub async fn get_users_bad_location () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let ha_client_keys = common::get_ha_client_keys();
    let server_key = common::get_pub_server_key();

    let users_res =
        obtain_users_at_location(
            EPOCH,
            GRID_SIZE,
            POS_Y,
            server_url,
            ha_client_keys.sign_key(),
            &server_key[0],
            ha_client_keys.clients_public_keys(),
        ).await;

    assert!(users_res.is_err());
}

#[tokio::test]
#[ignore]
pub async fn get_users_not_existent_epoch () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let ha_client_keys = common::get_ha_client_keys();
    let server_key = common::get_pub_server_key();

    let users_res =
        obtain_users_at_location(
            N_EPOCHS,
            POS_X,
            POS_Y,
            server_url,
            ha_client_keys.sign_key(),
            &server_key[0],
            ha_client_keys.clients_public_keys(),
        ).await;

    assert!(users_res.is_err());
}

#[tokio::test]
#[ignore]
pub async fn get_users_invalid_signature () {
    let server_url : Uri = Uri::from_static("http://[::1]:50000");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let server_key = common::get_pub_server_key();

    let ha_client_keys = common::get_ha_client_keys();

    let users_res =
        obtain_users_at_location(
            EPOCH,
            POS_X,
            POS_Y,
            server_url,
            client_keys.sign_key(),
            &server_key[0],
            ha_client_keys.clients_public_keys(),
        ).await;

    assert!(users_res.is_err());
}