#Highly Dependable Location Tracker - SEC

Highly Dependable Location Tracker is a contact tracing system where users prove their location and witness the location of others around them. It is also composed of a client, the ha client that can query the server on any user's location at all times.

## Requirements
- Rust v1.51.0

## Structure

The system directories are organized as follows:
* client : contains the source code related to the client side;
* grid : contains the source code related to the grid used for the system;
* ha\_client : contains the source code related to the ha client side;
* multiple\_servers : (not used yet -> for the 2nd stage of the system);
* protos : contains the protos used for the gRPC communication;
* report : contains the report explaining the project and our decisions;
* sbin : contains the script to run the entire system;
* security : contains the source code related with the security of the system and a folder(keys) with all the keys for the system in it;
* single\_server : contains the source code related to the server side;
* target : contains the binaries;

## Compiling

`cargo build`

## Running unit testing(incomplete)
`cargo test -p client --test proofing_system_tests -- --ignored`

## Running bizantine client tests
The bizantine user test requires the test name, so you'll need to run one at a time. 

`cargo test -p client <test_name> -- --ignored`


## Running the system

All binaries will be in `./target/debug/`

It is possible to run each binary on its own, by using the name followed by -h to see the options, however if you do this the server needs to be started first. 
To avoid that tedious process you can use the script launch\_system.sh that starts the server and a number of clients. The script uses the gnome-terminal but if for some reason that is not your default terminal the only thing you need to do is change the terminal name in the script.

To run the script:
`./sbin/launch_system.sh`

If you desire to run everything on your own, you can start the server by: `./target/debug/single-server --fline <fline> --keys <keys> --server <server> --size <size> --storage <storage>` and then start each one of the clients individually.
