use std::{collections::BTreeSet, fs, net::Ipv4Addr, path::Path};

use ipnet::Ipv4AddrRange;
use pnet::util::MacAddr;
use toml::{Table, Value};

use crate::config;

pub struct Profile {
    pub interface_ip: Ipv4Addr,
    pub gateway_mac: MacAddr,
    pub ip_vec: Vec<Ipv4Addr>,
    pub ports_vec: Vec<u16>,
    pub show_open: bool,
    pub show_closed: bool,
    pub show_filtered: bool,
}
impl Profile {
    fn new(
        interface_ip: Ipv4Addr,
        gateway_mac: MacAddr,
        ip_vec: Vec<Ipv4Addr>,
        ports_vec: Vec<u16>,
        show_open: bool,
        show_closed: bool,
        show_filtered: bool,
    ) -> Self {
        Profile {
            interface_ip,
            gateway_mac,
            ip_vec,
            ports_vec,
            show_open,
            show_closed,
            show_filtered,
        }
    }
}

#[derive(Debug)]
struct ShowRule {
    open: bool,
    closed: bool,
    filtered: bool,
}
impl Default for ShowRule {
    fn default() -> Self {
        ShowRule {
            open: true,
            closed: false,
            filtered: false,
        }
    }
}

pub fn parse(path: impl AsRef<Path>) -> Profile {
    let table: Table = fs::read_to_string(path).unwrap().parse().unwrap();

    let (interface_ip, gateway_mac, show_rules) = parse_profile(&table);

    let (ip_vec, ports_vec) = parse_targets(&table);

    Profile::new(
        interface_ip,
        gateway_mac,
        ip_vec,
        ports_vec,
        show_rules.open,
        show_rules.closed,
        show_rules.filtered,
    )
}

fn parse_profile(table: &Table) -> (Ipv4Addr, MacAddr, ShowRule) {
    let profile = table.get("profile").unwrap();

    let Value::String(interface_ip) = profile.get("interface").unwrap().get("ip").unwrap() else {
        panic!("can not find interface ip");
    };
    let Value::String(gateway_mac) = profile.get("gateway").unwrap().get("mac").unwrap() else {
        panic!("can not find gateway mac");
    };

    let mut rule = ShowRule::default();

    if let Some(show_table) = profile.get("show") {
        let show_open = if let Some(open) = show_table.get("open") {
            open.as_bool().expect("show rules error: open!!")
        } else {
            false
        };

        let show_closed = if let Some(closed) = show_table.get("closed") {
            closed.as_bool().expect("show rules error: closed!!")
        } else {
            false
        };

        let show_filtered = if let Some(filtered) = show_table.get("filtered") {
            filtered.as_bool().expect("show rules error: filtered!!")
        } else {
            false
        };

        rule.open = show_open;
        rule.filtered = show_filtered;
        rule.closed = show_closed;
    }

    let interface_ip: Ipv4Addr = interface_ip.parse().unwrap();
    let gateway_mac: MacAddr = gateway_mac.parse().unwrap();

    (interface_ip, gateway_mac, rule)
}

fn parse_targets(table: &Table) -> (Vec<Ipv4Addr>, Vec<u16>) {
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

            Value::String(s) => {
                if s == "all" {
                    (0..=65535).for_each(|x| ports_vec.push(x));
                } else if s == "known" {
                    let ports_known = config::get_ports_known();
                    ports_vec.extend_from_slice(ports_known);
                } else {
                    panic!("Unsupported ports type");
                }
            }

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

    (ip_vec, ports_vec)
}
