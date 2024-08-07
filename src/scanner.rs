use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use colored::*;
use indicatif::{ProgressBar, ProgressStyle, WeakProgressBar};
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

pub fn scan(
    interface_ip: Ipv4Addr,
    gateway_mac: MacAddr,
    socket_addr: Vec<SocketAddrV4>,
) -> (Vec<SocketAddrV4>, Vec<SocketAddrV4>, Vec<SocketAddrV4>) {
    println!("{} {}", "💀", "START ICMP DETECTING: ".blue().bold());

    let interface = datalink::interfaces()
        .into_iter()
        .find(|x| x.ips.first().expect("interface ip error!").ip() == IpAddr::V4(interface_ip))
        .expect("can not find the interface!!");

    let interface_clone = interface.clone();
    let gateway_mac_clone = gateway_mac.clone();

    let sockets_btree = get_btree(&socket_addr);

    let pb = ProgressBar::new(socket_addr.len() as u64);
    pb.set_message("SCANNING");
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan/blue}{msg:.blue} [{elapsed}] [{bar:50.cyan/blue}]) [{pos}/{len}]",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    let rx_pb = pb.downgrade();
    let tx_pb = pb.downgrade();

    let rx_thread = thread::spawn(move || receive(interface, gateway_mac, sockets_btree, rx_pb));

    let tx_thread = thread::spawn(move || {
        send(interface_clone, gateway_mac_clone, socket_addr, tx_pb);
    });

    let (open_ports, closed_ports, filtered_ports) =
        rx_thread.join().expect("receive thread error!");
    let _ = tx_thread.join().expect("send thread error");

    pb.finish_with_message("💀 SCANNING DONE");

    (open_ports, closed_ports, filtered_ports)
}

fn get_btree(target_sockets: &[SocketAddrV4]) -> BTreeSet<SocketAddrV4> {
    target_sockets.iter().map(|x| *x).collect()
}

fn send(
    interface: NetworkInterface,
    gateway_mac: MacAddr,
    target_sockets: Vec<SocketAddrV4>,
    pb: WeakProgressBar,
) {
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type!"),
        Err(e) => panic!("Error happened: {}", e),
    };

    let IpAddr::V4(src_ip) = interface.ips.first().expect("interface ip error!").ip() else {
        panic!();
    };

    for dest_socket in target_sockets {
        let src_port = rand::thread_rng().gen_range(20000..=65535);

        let packet_syn = packet::build(
            interface.mac.expect("interface MAC error!"),
            SocketAddrV4::new(src_ip, src_port),
            dest_socket,
            gateway_mac,
            TcpFlags::SYN,
        );

        tx.send_to(&packet_syn, None).unwrap().unwrap();
        pb.upgrade().unwrap().inc(1);

        thread::sleep(Duration::from_micros(1));
    }

    thread::sleep(Duration::from_millis(100));

    DONE.store(true, Ordering::SeqCst);
}

fn receive(
    interface: NetworkInterface,
    gateway_mac: MacAddr,
    mut target_sockets: BTreeSet<SocketAddrV4>,
    pb: WeakProgressBar,
) -> (Vec<SocketAddrV4>, Vec<SocketAddrV4>, Vec<SocketAddrV4>) {
    let IpAddr::V4(src_ip) = interface.ips.first().expect("interface ip error!").ip() else {
        panic!();
    };

    let Ok(Channel::Ethernet(mut tx, mut rx)) = datalink::channel(&interface, Default::default())
    else {
        panic!()
    };

    let mut open_ports = Vec::new();
    let mut filtered_ports = Vec::new();
    let mut closed_ports = Vec::new();

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

                if is_ack_syn(tcp_flags) {
                    pb.upgrade().unwrap().println(format!(
                        "   {} {}",
                        "OPEN".green().bold(),
                        target_socket
                    ));
                    open_ports.push(target_socket);
                    target_sockets.remove(&target_socket);

                    let packet_rst = packet::build(
                        interface.mac.unwrap(),
                        SocketAddrV4::new(src_ip, tcp_packet.get_destination()),
                        target_socket,
                        gateway_mac,
                        TcpFlags::RST,
                    );

                    tx.send_to(&packet_rst, None).unwrap().unwrap();
                } else if is_rst(tcp_flags) {
                    closed_ports.push(target_socket);
                    target_sockets.remove(&target_socket);
                }
            }
        }

        if DONE.load(Ordering::SeqCst) {
            break;
        }
    }

    filtered_ports.append(&mut target_sockets.into_iter().collect::<Vec<SocketAddrV4>>());

    (open_ports, closed_ports, filtered_ports)
}

fn is_ack_syn(tcp_flags: u8) -> bool {
    (tcp_flags & TcpFlags::SYN != 0) && (tcp_flags & TcpFlags::ACK != 0)
}

fn is_rst(tcp_flags: u8) -> bool {
    tcp_flags & TcpFlags::RST != 0
}
