/*  This software is licensed under the GPLv3 license. Please see the license
 *  file in the root directory of this repository for more details.
 *
 *  Copyright (c) Martin Hübner, 2022
 */


use std::collections::BTreeMap;
use std::process::Command;
use std::net::IpAddr;
use std::{fmt, fs, str};

use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct HNAjson {
    gateway: IpAddr,
    destination: IpAddr,
    genmask: u8,
    validityTime: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct OLSRjson {
    pid: u16,
    systemTime: u64,
    timeSinceStartup: u64,
    configurationChecksum: String,
    hna: Vec<HNAjson>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct CidrAddr {
    netaddr: IpAddr,
    netmask: u8,
}

impl fmt::Display for CidrAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.netaddr, self.netmask)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct HNAData {
    gateway: IpAddr,
    hna: CidrAddr,
    ttl: u32,
    host_name: String,
}


fn read_hna_to_tree(tree: &mut BTreeMap<IpAddr, HNAData>, raw_data: &str) {
    let d: OLSRjson = serde_json::from_str(raw_data).unwrap();

    for obj in d.hna {
        tree.insert(
            obj.destination,
            HNAData {
                gateway: obj.gateway,
                hna: CidrAddr {
                    netaddr: obj.destination,
                    netmask: obj.genmask,
                },
                ttl: obj.validityTime,
                host_name: "".to_string(),
            },
        );
    }

    ()
}

fn read_hosts_to_tree(tree: &mut BTreeMap<IpAddr, String>, raw_data: String) {
    let lines = raw_data.lines(); //.skip(2);

    for line in lines {
        let split: Vec<&str> = line.split_whitespace().collect();

        // ignore empty lines
        if split.len() < 2 {
            continue;
        }

        // ignore commenting lines (starting with '#')
        if split[0].starts_with("#") {
            continue;
        }

        let gw_ip: IpAddr = split[0].parse().unwrap();
        let hostname: String = split[1].parse().unwrap();

        tree.insert(gw_ip, hostname);
    }
}

fn main() {

    let hna4_json_raw = Command::new("sh")
        .arg("-c")
        .arg("echo /hna | nc 127.0.0.1 9090")
        .output()
        .expect("failed to execute process");
    let hna4_json = str::from_utf8(&hna4_json_raw.stdout).unwrap();

    // TODO: add IPv6-stuff
    // let hna6_raw = fs::read_to_string("raw/hna6_2006.txt").unwrap();

    // let hostnames_raw = fs::read_to_string("raw/olsr.txt").unwrap();
    let hostnames_raw = fs::read_to_string("/tmp/hosts/olsr").unwrap();

    let mut hna_tree = BTreeMap::new();
    let mut name_tree = BTreeMap::new();

    read_hna_to_tree(&mut hna_tree, &hna4_json);
    read_hosts_to_tree(&mut name_tree, hostnames_raw);

    // merge hostnames to hna
    for (_key, item) in &mut hna_tree {
        if name_tree.contains_key(&item.gateway) {
            let hostname = name_tree.get(&item.gateway).unwrap();

            item.host_name = hostname.to_string();
        }
    }

    // print results in a nicely formatted way.
    println!("Announced network      OLSR gateway      Validity Time      OLSR Hostname");
    println!("=================      ============      =============      =============");

    for (_key, val) in hna_tree.iter() {
        println!(
            "{:<22} {:<20} {:<15} {:<25}",
            // formatting doesn't work properly, if we don't add the string cast...
            val.hna.to_string(),
            val.gateway.to_string(),
            // show seconds
            val.ttl / 1000,
            val.host_name
        );
    }
}
