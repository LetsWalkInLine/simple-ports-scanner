mod toml_parser;

fn main() {
    let (interface_ip, gateway_mac, socket_addr) = toml_parser::parse("example/test.toml");

    println!("{}", interface_ip);
    println!("{}", gateway_mac);
    println!("{:?}", socket_addr);

}
