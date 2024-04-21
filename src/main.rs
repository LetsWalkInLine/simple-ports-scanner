mod config;
mod display;
mod icmp_detector;
mod scanner;
mod toml_parser;

use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
};

fn main() {
    let mut args = env::args().skip(1);
    let Some(profile_path) = args.next() else {
        show_usage();
        return;
    };
    let output_path = args.next().unwrap_or(String::from("output.toml"));

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

fn show_usage() {
    println!(
        "
USAGE: syn_port_scanner [-p profile_path] [-o output_path]
    
OPTIONS:
    -p profile_path     执行配置文件的路径
    -o output_path      输出结果文件的路径名，默认值为当前目录，
                        输出格式为toml格式"
    );
}
