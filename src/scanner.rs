use std::{
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use pnet::{
    datalink::{self, Channel, NetworkInterface},
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

static DONE: AtomicBool = AtomicBool::new(false);

pub fn test(interface_ip: Ipv4Addr, gateway_mac: MacAddr, socket_addr: Vec<SocketAddrV4>) {
    let interface = datalink::interfaces()
        .into_iter()
        .find(|x| x.ips.first().unwrap().ip() == IpAddr::V4(interface_ip))
        .unwrap();

    let interface_clone = interface.clone();
    let gateway_mac_clone = gateway_mac.clone();
    let socket_addr_clone = socket_addr.clone();

    let rx_thread = thread::spawn(move || {
        receive(interface, gateway_mac, socket_addr);
    });

    let tx_thread = thread::spawn(move || {
        send(interface_clone, gateway_mac_clone, socket_addr_clone);
    });

    let _ = rx_thread.join().unwrap();
    let _ = tx_thread.join().unwrap();
}

fn send(interface: NetworkInterface, gateway_mac: MacAddr, target_sockets: Vec<SocketAddrV4>) {
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type!"),
        Err(e) => panic!("Error happened: {}", e),
    };

    for dest_socket in target_sockets {
        let IpAddr::V4(src_ip) = interface.ips.first().unwrap().ip() else {
            panic!();
        };

        let src_port = rand::thread_rng().gen_range(20000..=65535);

        let packet_syn = packet::build(
            interface.mac.unwrap(),
            SocketAddrV4::new(src_ip, src_port),
            dest_socket,
            gateway_mac,
            TcpFlags::SYN,
        );

        tx.send_to(&packet_syn, None).unwrap().unwrap();
    }

    thread::sleep(Duration::from_secs(1));

    DONE.store(true, Ordering::SeqCst);
}

fn receive(interface: NetworkInterface, gateway_mac: MacAddr, target_sockets: Vec<SocketAddrV4>) {
    let IpAddr::V4(src_ip) = interface.ips.first().unwrap().ip() else {
        panic!();
    };

    let Ok(Channel::Ethernet(mut tx, mut rx)) = datalink::channel(&interface, Default::default())
    else {
        panic!()
    };

    loop {
        let eth_packet = EthernetPacket::new(rx.next().unwrap()).unwrap();
        let ipv4_packet = Ipv4Packet::new(eth_packet.payload()).unwrap();

        if ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp
            && ipv4_packet.get_destination() == src_ip
        {
            let tcp_packet = TcpPacket::new(ipv4_packet.payload()).unwrap();

            let target_ip = ipv4_packet.get_source();
            let target_port = tcp_packet.get_source();
            let target_socket = SocketAddrV4::new(target_ip, target_port);

            if target_sockets.contains(&target_socket) {
                let tcp_flags = tcp_packet.get_flags();

                if (tcp_flags & TcpFlags::SYN != 0) && (tcp_flags & TcpFlags::ACK != 0) {
                    println!("OPEN: {}", target_socket);

                    let packet_rst = packet::build(
                        interface.mac.unwrap(),
                        SocketAddrV4::new(src_ip, tcp_packet.get_destination()),
                        target_socket,
                        gateway_mac,
                        TcpFlags::RST,
                    );

                    tx.send_to(&packet_rst, None).unwrap().unwrap();
                } else if tcp_flags & TcpFlags::RST != 0 {
                    // println!("CLOSE: {}", target_socket);
                } else {
                    println!("Ooops");
                }
            }
        }

        if DONE.load(Ordering::SeqCst) {
            break;
        }
    }
}
