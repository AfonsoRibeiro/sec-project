mod verifying;

use std::{collections::{HashMap, HashSet}, sync::Arc};

use futures::stream::{FuturesUnordered, StreamExt};
use futures::select;

use structopt::StructOpt;
use color_eyre::eyre::Result;
use regex::Regex;

use tonic::transport::Uri;
use tokio::io::{self, AsyncBufReadExt, BufReader};

use security::key_management::{HAClientKeys, ServerPublicKey, retrieve_ha_client_keys, retrieve_servers_public_keys};

#[derive(StructOpt)]
#[structopt(name = "HA_Client", about = "Checking on server satus")]
struct Opt {

    #[structopt(name = "n_servers", long, default_value = "1")]
    n_servers : usize,

    #[structopt(name = "size", long, default_value = "5")]
    grid_size : usize,

    #[structopt(name = "keys", long, default_value = "security/keys")]
    keys_dir : String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Starting HA_Client");

    let opt = Opt::from_args();

    let f_servers = (opt.n_servers - 1) / 3;
    let necessary_res= f_servers + opt.n_servers / 2;

    let ha_keys = retrieve_ha_client_keys(&opt.keys_dir)?;
    let server_keys = retrieve_servers_public_keys(&opt.keys_dir)?;

    let server_urls  = get_servers_url(opt.n_servers);

    sodiumoxide::init().expect("Unable to make sodiumoxide thread safe");

    read_commands(opt.grid_size, server_urls, &ha_keys, &server_keys, necessary_res, f_servers).await;

    Ok(())
}

async fn do_report_command(
    grid_size : usize,
    server_urls :  Arc<Vec<Uri>>,
    ha_keys : &HAClientKeys,
    server_keys : &ServerPublicKey,
    f_servers : usize,
    idx : usize,
    epoch : usize,
) {

    let client_pub_key = ha_keys.client_public_key(idx);

    if client_pub_key.is_none() { println!("Invalid idx for client"); return; }
    let client_pub_key = client_pub_key.unwrap();

    let mut responses : FuturesUnordered<_> = server_urls.iter().enumerate().map(
        |(server_id, url)|
            verifying::obtain_location_report(
                idx,
                epoch,
                grid_size,
                url.clone(),
                ha_keys.sign_key(),
                server_keys.public_key(server_id),
                client_pub_key,
            )
        ).collect();

    let mut locations : HashMap<(usize, usize), usize> = HashMap::new();
    let mut location : (usize, usize) = (usize::MAX, usize::MAX);
    let mut max : usize = 0;
    loop {
        select! {
            res = responses.select_next_some() => {
                if let Ok(loc) = res {
                    match locations.get_mut(&loc) {
                        Some(n) => {
                            *n += 1;
                            if *n > max {
                                max = *n;
                                location = loc;
                            }
                        }
                        None => {
                            if max == 0 {
                                max = 1;
                                location = loc.clone();
                            }
                            locations.insert(loc, 1);
                        }
                    };
                }


                if max > f_servers {
                    println!("{:?}" ,location);
                    break ;
                }
            }
            complete => break,
        }
    }
}

async fn do_get_users_at_loc_command(
    server_urls :  Arc<Vec<Uri>>,
    ha_keys : &HAClientKeys,
    server_keys : &ServerPublicKey,
    necessary_res : usize,
    epoch : usize,
    pos_x : usize,
    pos_y : usize,
) {

    let mut responses : FuturesUnordered<_> = server_urls.iter().enumerate().map(
        |(server_id, url)|
            verifying::obtain_users_at_location(
                epoch,
                pos_x,
                pos_y,
                url.clone(),
                ha_keys.sign_key(),
                server_keys.public_key(server_id),
                ha_keys.clients_public_keys()
            )
        ).collect();

    let mut all_users : Vec<HashSet<usize>> = Vec::with_capacity(necessary_res + 1);
    loop {
        select! {
            res = responses.select_next_some() => {
                if let Ok(users) = res {
                    all_users.push(users);
                }

                if all_users.len() > necessary_res {
                    break ;
                }
            }
            complete => break,
        }
    }
    let mut max_occurencies : usize = 0;
    let mut max_occurencies_set = HashSet::new();
    for i in 0..((all_users.len() + 1)/2) {
        let mut occurencies : usize = 1;
        for j in (i+1)..all_users.len() {
            if all_users[i].len() == all_users[j].len(){
                for user in all_users[i].iter(){
                    if !all_users[j].contains(user) {
                        continue;
                    }
                }
                occurencies += 1;
            }
        }
        if occurencies > max_occurencies {
            max_occurencies = occurencies;
            max_occurencies_set = all_users[i].clone();
        }
    }

    println!("{:?}", max_occurencies_set);

}

async fn read_commands(
    grid_size : usize,
    server_urls :  Arc<Vec<Uri>>,
    ha_keys : &HAClientKeys,
    server_keys : &ServerPublicKey,
    necessary_res : usize,
    f_servers : usize,
) {
    print_command_msg();

    let o_rep_pat = Regex::new(r"r(eport)? [+]?(\d+) [+]?(\d+)").unwrap();
    let o_users_pat = Regex::new(r"u(sers)? [+]?(\d+) [+]?(\d+) [+]?(\d+)").unwrap();

    let mut reader = BufReader::new(io::stdin());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        reader.read_line(&mut buffer).await.unwrap();
        {
            if let Some(cap) = o_rep_pat.captures(buffer.trim_end()) {
                let idx  = cap[2].parse::<usize>();
                let epoch  = cap[3].parse::<usize>();
                if idx.is_err() || epoch.is_err() { print_command_msg(); continue; }

                do_report_command(
                    grid_size,
                    server_urls.clone(),
                    ha_keys,
                    server_keys,
                    f_servers,
                    idx.unwrap(),
                    epoch.unwrap()
                ).await;

            } else if let Some(cap) = o_users_pat.captures(buffer.trim_end()) {
                let epoch  = cap[2].parse::<usize>();
                let pos_x  = cap[3].parse::<usize>();
                let pos_y  = cap[4].parse::<usize>();
                if epoch.is_err() || pos_x.is_err() || pos_y.is_err() { print_command_msg(); continue; }

                do_get_users_at_loc_command(
                    server_urls.clone(),
                    ha_keys,
                    server_keys,
                    necessary_res,
                    epoch.unwrap(),
                    pos_x.unwrap(),
                    pos_y.unwrap(),
                ).await

            } else {
                print_command_msg();
            }
        }
    }
}

fn print_command_msg() { println!("To obtain a report use: report <id> <epoch>\nTo obtain users ate location use: users <epoch> <pos_x> <pos_y>"); }

fn get_servers_url(n_servers : usize ) -> Arc<Vec<Uri>> {
    let mut server_urls = vec![];
    for i in 0..n_servers{
        server_urls.push(format!("http://[::1]:500{:02}", i).parse().unwrap());
    }
    Arc::new(server_urls)
}