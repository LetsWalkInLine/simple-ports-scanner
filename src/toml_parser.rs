use std::{collections::BTreeSet, fs, net::Ipv4Addr, path::Path};

use ipnet::Ipv4AddrRange;
use pnet::util::MacAddr;
use toml::{Table, Value};

pub fn parse(path: impl AsRef<Path>) -> (Ipv4Addr, MacAddr, Vec<Ipv4Addr>, Vec<u16>) {
    let table: Table = fs::read_to_string(path).unwrap().parse().unwrap();

    let profile = table.get("profile").unwrap();

    let Value::String(interface_ip) = profile.get("interface").unwrap().get("ip").unwrap() else {
        panic!("can not find interface ip");
    };
    let Value::String(gateway_mac) = profile.get("gateway").unwrap().get("mac").unwrap() else {
        panic!("can not find gateway mac");
    };
    let interface_ip: Ipv4Addr = interface_ip.parse().unwrap();

    let targets = table.get("target").unwrap().as_array().unwrap();

    let mut ip_vec: Vec<Ipv4Addr> = Vec::new();
    let mut ports_vec: Vec<u16> = Vec::new();

    for item in targets {
        let ip = item.get("ip").unwrap();
        let ports = item.get("ports").unwrap();

        match ip {
            Value::String(ip) => ip_vec.push(ip.parse().unwrap()),
            Value::Array(ips) => ips
                .iter()
                .for_each(|x| ip_vec.push(x.as_str().unwrap().parse().unwrap())),
            Value::Table(ips) => {
                let from: Ipv4Addr = ips.get("from").unwrap().as_str().unwrap().parse().unwrap();
                let to: Ipv4Addr = ips.get("to").unwrap().as_str().unwrap().parse().unwrap();
                Ipv4AddrRange::new(from, to)
                    .filter(|x| !x.is_broadcast() && !x.is_multicast())
                    .for_each(|x| ip_vec.push(x));
            }
            _ => panic!("Unsupported Ip type"),
        }

        match ports {
            Value::Integer(port) => ports_vec.push(*port as u16),
            Value::Array(ports) => ports
                .iter()
                .for_each(|x| ports_vec.push(x.as_integer().unwrap() as u16)),
            Value::Table(ports) => {
                let from = ports.get("from").unwrap().as_integer().unwrap() as u16;
                let to = ports.get("to").unwrap().as_integer().unwrap() as u16;
                (from..=to).for_each(|x| ports_vec.push(x));
            }
            Value::String(_) => (0..=65535).for_each(|x| ports_vec.push(x)),
            _ => panic!("Unsupported ports type"),
        }
    }

    let ip_vec = ip_vec
        .into_iter()
        .collect::<BTreeSet<Ipv4Addr>>()
        .into_iter()
        .collect();

    let ports_vec = ports_vec
        .into_iter()
        .collect::<BTreeSet<u16>>()
        .into_iter()
        .collect();

    (
        interface_ip,
        gateway_mac.parse().unwrap(),
        ip_vec,
        ports_vec,
    )
}
