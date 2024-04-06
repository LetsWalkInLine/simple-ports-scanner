mod toml_parser;
mod scanner;

fn main() {
    let (interface_ip, gateway_mac, socket_addr) = toml_parser::parse("example/test.toml");

    println!("{}", interface_ip);
    println!("{}", gateway_mac);

    scanner::test(interface_ip, gateway_mac, socket_addr);

}
