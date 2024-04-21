mod packet;

use indicatif::{ProgressBar, ProgressStyle, WeakProgressBar};
use pnet::{
    datalink::{self, Channel, NetworkInterface},
    packet::{
        ethernet::EthernetPacket,
        icmp::{IcmpPacket, IcmpTypes},
        ip::IpNextHeaderProtocols,
        ipv4::Ipv4Packet,
        Packet,
    },
    util::MacAddr,
};

use colored::*;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

static DONE: AtomicBool = AtomicBool::new(false);

pub fn detect(
    interface_ip: Ipv4Addr,
    gateway_mac: MacAddr,
    dest_ips: Vec<Ipv4Addr>,
) -> Vec<Ipv4Addr> {
    println!("{} {}", "😁", "START ICMP DETECTING: ".yellow().bold());

    let interface = datalink::interfaces()
        .into_iter()
        .find(|x| x.ips.first().unwrap().ip() == IpAddr::V4(interface_ip))
        .unwrap();

    let interface_clone = interface.clone();
    let gateway_mac_clone = gateway_mac.clone();
    let dest_ip_clone = dest_ips.clone();

    let pb = ProgressBar::new(dest_ips.len() as u64);
    pb.set_message("DETECTING");
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan/blue}{msg:.yellow} [{elapsed}] [{bar:50.cyan/blue}]) [{pos}/{len}]",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    let rx_pb = pb.downgrade();
    let tx_pb = pb.downgrade();

    let rx_thread = thread::spawn(move || receive_and_filter(interface, dest_ips, rx_pb));

    let tx_thread = thread::spawn(move || {
        send(interface_clone, gateway_mac_clone, dest_ip_clone, tx_pb);
    });

    let reachable_ips = rx_thread.join().unwrap();
    let _ = tx_thread.join().unwrap();

    pb.finish_with_message("😁 DETECTING DONE ");

    reachable_ips
}

fn send(
    interface: NetworkInterface,
    gateway_mac: MacAddr,
    target_dests: Vec<Ipv4Addr>,
    pb: WeakProgressBar,
) {
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type!"),
        Err(e) => panic!("Error happened: {}", e),
    };

    let IpAddr::V4(src_ip) = interface.ips.first().unwrap().ip() else {
        panic!();
    };

    let interface_mac = interface.mac.unwrap();

    for dest_ip in target_dests {
        let packet_icmp = packet::build(interface_mac, src_ip, dest_ip, gateway_mac);

        tx.send_to(&packet_icmp, None).unwrap().unwrap();
        pb.upgrade().unwrap().inc(1);

        thread::sleep(Duration::from_micros(1));
    }

    thread::sleep(Duration::from_millis(100));

    DONE.store(true, Ordering::SeqCst);
}

fn receive_and_filter(
    interface: NetworkInterface,
    target_dests: Vec<Ipv4Addr>,
    pb: WeakProgressBar,
) -> Vec<Ipv4Addr> {
    let IpAddr::V4(src_ip) = interface.ips.first().unwrap().ip() else {
        panic!();
    };

    let Ok(Channel::Ethernet(_, mut rx)) = datalink::channel(&interface, Default::default()) else {
        panic!()
    };

    let mut reachable_ips = Vec::with_capacity(target_dests.len());

    loop {
        let raw_packet = rx.next().unwrap();
        let eth_packet = EthernetPacket::new(raw_packet).unwrap();
        let ipv4_packet = Ipv4Packet::new(eth_packet.payload()).unwrap();

        if ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Icmp
            && ipv4_packet.get_destination() == src_ip
            && target_dests.contains(&ipv4_packet.get_source())
        {
            let icmp_packet = IcmpPacket::new(ipv4_packet.payload()).unwrap();

            if icmp_packet.get_icmp_type() == IcmpTypes::EchoReply {
                let from = ipv4_packet.get_source();

                pb.upgrade()
                    .unwrap()
                    .println(format!("{} {}", "REACHABLE".green().bold(), from));

                reachable_ips.push(from);
            }
        }

        if DONE.load(Ordering::SeqCst) {
            break;
        }
    }

    reachable_ips
}
