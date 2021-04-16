mod common;

/// Requires ./sbin/proofing_test_setup.sh

use client::{proofing_system, reports};
use security::report::Report;
use security::proof::{Proof, sign_proof};
use tonic::transport::Uri;

use std::iter::FromIterator;

const IDX : usize = 19;
const EPOCH : usize = 1;
const SIZE : usize = 3;

#[tokio::test]
#[ignore]
pub async fn submit_correct_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline, IDX, EPOCH).await;
        if proofs.len() > 0 && proofs.len() == idxs_ass.len() {
            let report = Report::new(EPOCH, (loc_x, loc_y), IDX, idxs_ass, proofs);
            assert!(
                reports::submit_location_report(
                    IDX,
                    &report,
                    server_url.clone(),
                    client_keys.sign_key(),
                    sever_key,
                ).await.is_ok()
            );

        } else {
            panic!("Client {:} unable to generate report for epoch {:}.", IDX, EPOCH);
        }
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}

#[tokio::test]
#[ignore]
pub async fn submit_correct_report_twice () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline, IDX, EPOCH).await;
        if proofs.len() > 0 && proofs.len() == idxs_ass.len() {
            let report = Report::new(EPOCH, (loc_x, loc_y), IDX, idxs_ass.clone(), proofs.clone());
            assert!(
                reports::submit_location_report(
                    IDX,
                    &report,
                    server_url.clone(),
                    client_keys.sign_key(),
                    sever_key,
                ).await.is_ok()
            );
            let report = Report::new(EPOCH, (loc_x, loc_y), IDX, idxs_ass, proofs);
            assert!(
                reports::submit_location_report(
                    IDX,
                    &report,
                    server_url.clone(),
                    client_keys.sign_key(),
                    sever_key,
                ).await.is_ok()
            );

        } else {
            panic!("Client {:} unable to generate report for epoch {:}.", IDX, EPOCH);
        }
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}

#[tokio::test]
#[ignore]
pub async fn submit_empty_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let proofs = vec![];
        let idxs_ass = vec![];
        let report = Report::new(EPOCH, (loc_x, loc_y), IDX, idxs_ass, proofs);
        assert!(
            reports::submit_location_report(
                IDX,
                &report,
                server_url.clone(),
                client_keys.sign_key(),
                sever_key,
            ).await.is_err()
        );
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}

#[tokio::test]
#[ignore]
pub async fn submit_bad_location_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline, IDX, EPOCH).await;
        if proofs.len() > 0 && proofs.len() == idxs_ass.len() {
            let report = Report::new(EPOCH, ((loc_x+2)%SIZE, (loc_y+2)%SIZE), IDX, idxs_ass, proofs);
            assert!(
                reports::submit_location_report(
                    IDX,
                    &report,
                    server_url.clone(),
                    client_keys.sign_key(),
                    sever_key,
                ).await.is_ok()
            );

        } else {
            panic!("Client {:} unable to generate report for epoch {:}.", IDX, EPOCH);
        }
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}

#[tokio::test]
#[ignore]
pub async fn submit_only_my_proof_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");


    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let proof = Proof::new(EPOCH, IDX, IDX, (loc_x, loc_y));
        let proofs = vec![sign_proof(&client_keys.sign_key(), proof)];
        let idxs_ass = vec![IDX];
        let report = Report::new(EPOCH, (loc_x, loc_y), IDX, idxs_ass, proofs);
        assert!(
            reports::submit_location_report(
                IDX,
                &report,
                server_url.clone(),
                client_keys.sign_key(),
                sever_key,
            ).await.is_err()
        );
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}

#[tokio::test]
#[ignore]
pub async fn submit_not_enough_proofs_report () {
    let server_url : Uri = Uri::from_static("http://[::1]:50051");

    common::make_thread_safe();

    let client_keys = common::get_client_keys(IDX);
    let sever_key = common::get_pub_server_key();

    let timeline = common::get_timeline();
    if let Some((loc_x, loc_y)) = timeline.get_location_at_epoch(IDX, EPOCH) {
        let (proofs, idxs_ass) = proofing_system::get_proofs(timeline.clone(), IDX, EPOCH).await;
        let less_proos = Vec::from_iter(proofs[..timeline.f_line].iter().cloned());
        let less_idxs_ass = Vec::from_iter(idxs_ass[..timeline.f_line].iter().cloned());

        if proofs.len() > 0 && proofs.len() == idxs_ass.len() {
            let report = Report::new(EPOCH, (loc_x, loc_y), IDX, less_idxs_ass, less_proos);
            assert!(
                reports::submit_location_report(
                    IDX,
                    &report,
                    server_url.clone(),
                    client_keys.sign_key(),
                    sever_key,
                ).await.is_err()
            );

        } else {
            panic!("Client {:} unable to generate report for epoch {:}.", IDX, EPOCH);
        }
    } else {
        panic!("Error: reports_generator! (Should never happen)");
    }
}
