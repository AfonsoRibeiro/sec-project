# Highly Dependable Location Tracker - SEC

Highly Dependable Location Tracker is a contact tracing system where users prove their location and witness the location of others around them. It is also composed of a client, the ha client that can query the server on any user's location at all times.

## Requirements
- Rust v1.51.0

## Structure

The system directories are organized as follows:
* client : contains the source code related to the client side;
* grid : contains the source code related to the grid used for the system;
* ha\_client : contains the source code related to the ha client side;
* protos : contains the protos used for the gRPC communication;
* report : contains the report explaining the project and our decisions;
* sbin : contains the script to run the entire system;
* security : contains the source code related with the security of the system;
* server : contains the source code related to the server side;
* target : contains the binaries;

## Compiling

`cargo build`

## Running unit testing
`cargo test`

## Running integration tests
`./sbin/integration_tests_setup.sh`

`cargo test -p client -p ha_client --test proofing_system_tests --test obtain_report_tests --test verifying_tests -- --ignored`

## Running report related integration tests
These tests require you to rerun the set up script before running each test.

`./sbin/integration_tests_setup.sh`

`cargo test -p client <test_name> -- --ignored`

The test names are:
* submit_correct_report
* submit_empty_report
* submit_bad_location_report
* submit_only_my_proof_report
* submit_not_enough_proofs_report

## Running the system

All binaries will be in `./target/debug/`

It is possible to run each binary on its own, by using the name followed by -h to see the options, however if you do this the server needs to be started first.
To avoid that tedious process you can use the script launch\_system.sh that starts the server and a number of clients. The script uses the gnome-terminal but if for some reason that is not your default terminal the only thing you need to do is change the terminal name in the script.

To run the script:
`./sbin/launch_system.sh`

Each client is able to obtain his location report with the command:

`report <epoch>` (the ha client is able to obtain everyones reports)

`proof <epoch> <epoch>*`

The ha client has another command besides the one mentioned previously, which allows him to obtain the list of users at a position:

`users <epoch> <pos_x> <pos_y>`


