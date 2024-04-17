use std::net::{Ipv4Addr, SocketAddrV4};

mod config;
mod icmp_detector;
mod scanner;
mod toml_parser;

fn main() {
    let (interface_ip, gateway_mac, dest_ips, dest_ports) = toml_parser::parse("example/test.toml");

    icmp_detector::detect(interface_ip, gateway_mac, dest_ips.clone());

    let socket_addr = get_socket_addr(&dest_ips, &dest_ports);
    scanner::scan(interface_ip, gateway_mac, socket_addr);
}

fn get_socket_addr(dest_ips: &[Ipv4Addr], dest_ports: &[u16]) -> Vec<SocketAddrV4> {
    let mut pairs = Vec::with_capacity(dest_ips.len() * dest_ports.len());

    for ip in dest_ips {
        for port in dest_ports {
            pairs.push(SocketAddrV4::new(*ip, *port));
        }
    }

    pairs
}
