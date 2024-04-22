mod cli;
mod config;
mod display;
mod icmp_detector;
mod scanner;
mod toml_parser;

use std::net::{Ipv4Addr, SocketAddrV4};

fn main() {
    let (profile_path, output_path) = cli::get_args();

    // let profile = toml_parser::parse("example/test.toml");
    let profile = toml_parser::parse(profile_path);

    let reachable_ips = icmp_detector::detect(
        profile.interface_ip,
        profile.gateway_mac,
        profile.ip_vec.clone(),
    );

    let socket_addr = get_socket_addr(&reachable_ips, &profile.ports_vec);

    let (open_ports, closed_ports, filtered_ports) =
        scanner::scan(profile.interface_ip, profile.gateway_mac, socket_addr);

    display::display(
        open_ports,
        closed_ports,
        filtered_ports,
        profile.show_open,
        profile.show_closed,
        profile.show_filtered,
        output_path,
    );
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
