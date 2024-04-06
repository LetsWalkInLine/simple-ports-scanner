use std::{
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    time::{Duration, Instant},
};

use pnet::{
    datalink::{self, Channel},
    packet::{
        ethernet::EthernetPacket,
        ip::IpNextHeaderProtocols,
        ipv4::Ipv4Packet,
        tcp::{TcpFlags, TcpPacket},
        Packet,
    },
    util::MacAddr,
};
use rand::Rng;

mod packet;

pub fn test(interface_ip: Ipv4Addr, gateway_mac: MacAddr, socket_addr: Vec<SocketAddrV4>) {
    let interface = datalink::interfaces()
        .into_iter()
        .find(|x| x.ips.first().unwrap().ip() == IpAddr::V4(interface_ip))
        .unwrap();

    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type!"),
        Err(e) => panic!("Error happened: {}", e),
    };

    for dest_socket in socket_addr {
        let src_port = rand::thread_rng().gen_range(20000..=65535);

        let packet = packet::build(
            interface.mac.unwrap(),
            SocketAddrV4::new(interface_ip, src_port),
            dest_socket,
            gateway_mac,
            TcpFlags::SYN,
        );

        tx.send_to(&packet, None).unwrap().unwrap();

        let Ok(Channel::Ethernet(_, mut rx)) = datalink::channel(&interface, Default::default())
        else {
            panic!()
        };

        let start_time = Instant::now();
        loop {
            let eth_packet = EthernetPacket::new(rx.next().unwrap()).unwrap();
            let ipv4_packet = Ipv4Packet::new(eth_packet.payload()).unwrap();
            if ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp
                && ipv4_packet.get_destination() == interface_ip
                && ipv4_packet.get_source() == *dest_socket.ip()
            {
                let tcp_packet = TcpPacket::new(ipv4_packet.payload()).unwrap();

                if tcp_packet.get_source() == dest_socket.port()
                    && tcp_packet.get_destination() == src_port
                {
                    let tcp_flags = tcp_packet.get_flags();

                    if (tcp_flags & TcpFlags::SYN != 0) && (tcp_flags & TcpFlags::ACK != 0) {
                        println!("OPEN: {}", dest_socket);
                        break;
                    } else if tcp_flags & TcpFlags::RST != 0 {
                        println!("CLOSE: {}", dest_socket);
                        break;
                    } else {
                        println!("Ooops");
                    }
                }
            } else if Instant::now().duration_since(start_time) > Duration::from_micros(500) {
                println!("Unknown: {}", dest_socket);
                break;
            }
        }
    }
}
