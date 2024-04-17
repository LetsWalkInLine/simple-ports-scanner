mod config;
mod icmp_detector;
mod scanner;
mod toml_parser;

fn main() {
    let (interface_ip, gateway_mac, socket_addr) = toml_parser::parse("example/test.toml");

    // icmp_detector::test(interface_ip, gateway_mac, socket_addr);
    // scanner::test(interface_ip, gateway_mac, socket_addr);
}
