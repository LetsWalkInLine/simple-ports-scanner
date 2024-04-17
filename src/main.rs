mod config;
mod icmp_detector;
mod scanner;
mod toml_parser;

use std::collections::HashSet;

fn main() {
    let (interface_ip, gateway_mac, socket_addr) = toml_parser::parse("example/test.toml");

    let dest_ips: Vec<_> = socket_addr
        .iter()
        .map(|x| x.ip().clone())
        .collect::<HashSet<_>>()
        .iter()
        .map(|x| x.clone())
        .collect();

    icmp_detector::detect(interface_ip, gateway_mac, dest_ips);
    // scanner::scan(interface_ip, gateway_mac, socket_addr);
}
