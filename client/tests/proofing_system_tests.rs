mod common;

/// Requires ./sbin/proofing_test_setup.sh

use client::proofing_system::request_location_proof;
use security::proof;

const IDX : usize = 19;
const EPOCH : usize = 1;
const N_EPOCHS : usize = 10;

#[tokio::test]
#[ignore]
pub async fn get_proof () {
    let timeline = common::get_timeline();

    let neighbours = timeline.get_neighbours_at_epoch(IDX, EPOCH);

    let id_proofer = neighbours.expect("Need a neighbour to request a proof")[0];

    let (sign_proof, idx_ass) = request_location_proof(IDX, EPOCH, id_proofer).await.unwrap();

    assert_eq!(id_proofer, idx_ass as usize,"Proofer id does not match id of the responder");

    assert_eq!(id_proofer, idx_ass as usize);

    let sign_pk = common::get_pub_sign_key(id_proofer);

    // Validating recieved correct proof
    let poof = proof::verify_proof(&sign_pk, &sign_proof).expect("Unhable to verify proof");

    assert_eq!(IDX, poof.idx_req());
    assert_eq!(id_proofer, poof.idx_ass());
    assert_eq!(EPOCH, poof.epoch());
    assert_eq!(timeline.get_location_at_epoch(id_proofer, EPOCH).unwrap(), poof.loc_ass());
}

#[tokio::test]
#[ignore]
pub async fn bad_id_get_proof () {
    let timeline = common::get_timeline();

    let neighbours = timeline.get_neighbours_at_epoch(IDX, EPOCH);

    let id_proofer = neighbours.expect("Need a neighbour to request a proof")[0];

    request_location_proof(IDX*5, EPOCH, id_proofer).await.expect_err("Got a proof, when i shouln't have");
}

#[tokio::test]
#[ignore]
pub async fn bad_epoch_get_proof () {
    request_location_proof(IDX, N_EPOCHS, 5).await.expect_err("Got a proof, when i shouln't have");
    request_location_proof(IDX, N_EPOCHS + 2, 6).await.expect_err("Got a proof, when i shouln't have");
}

