/*  This software is licensed under the GPLv3 license. Please see the license
 *  file in the root directory of this repository for more details.
 *
 *  Copyright (c) Martin Hübner, 2022
 */

use anyhow::{Context, Ok, Result};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::{env, fmt, fs, str, time::Duration};

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
    let d: OLSRjson =
        serde_json::from_str(raw_data).expect("Wasn't able to parse OLSR JSON-String correctly.");

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
}

fn read_hosts_to_tree(tree: &mut BTreeMap<IpAddr, String>, raw_data: String) {
    // split in lines, then filter empty lines and comments away,
    // from the remaining lines, parse hostname and IPAddr
    let lines = raw_data
        .lines()
        .filter(|i| !i.starts_with('#'))
        .filter(|i| !i.is_empty())
        .map(|x| {
            let mut split = x.split_whitespace();

            let gw_ip: IpAddr = split.next().unwrap().parse().unwrap();
            let hostname: String = split.next().unwrap().parse().unwrap();

            return (gw_ip, hostname);
        });

    // set funnel on tree and let iterator-result run in...
    tree.extend(lines);
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let host_addr: IpAddr = if args.len() > 1 {
        args[1]
            .parse()
            .context("You didn't give a valid IP address. Please only give addresses, no hostnames.")?
    } else {
        "127.0.0.1".parse().unwrap()
    };

    let err_msg = "Wasn't able to open a socket to the OLSR-Daemon at ".to_owned()
        + &host_addr.to_string()
        + ":9090.";

    let sock_addr = SocketAddr::new(host_addr, 9090);
    let mut conn = TcpStream::connect_timeout(&sock_addr, Duration::new(2, 0)).context(err_msg)?;
    conn.set_read_timeout(Some(Duration::new(5, 0)))
        .context("Setting connection timeout failed!")?;
    conn.write("/hna".as_bytes())
        .context("Wasn't able to write to the socket.")?;

    let mut hna4_json = "".to_string();
    conn.read_to_string(&mut hna4_json)
        .context("reading from the socket failed.")?;

    // TODO: add IPv6-stuff
    // let hna6_raw = fs::read_to_string("raw/hna6_2006.txt").unwrap();

    let hostnames_raw =
        fs::read_to_string("/tmp/hosts/olsr").context("Wasn't able to open '/tmp/hosts/olsr'.")?;

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

    Ok(())
}
