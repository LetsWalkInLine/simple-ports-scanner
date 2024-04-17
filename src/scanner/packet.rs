use std::net::SocketAddrV4;

use pnet::{
    packet::{
        ethernet::{EtherTypes, MutableEthernetPacket},
        ip::IpNextHeaderProtocols,
        ipv4::{checksum, Ipv4Flags, MutableIpv4Packet},
        tcp::{ipv4_checksum, MutableTcpPacket, TcpOption},
    },
    util::MacAddr,
};

use crate::config::{ETHERNET_HEADER_LEN, IPV4_HEADER_LEN};

pub fn build(
    src_mac: MacAddr,
    src_socket: SocketAddrV4,
    dest_socket: SocketAddrV4,
    gateway_mac: MacAddr,
    flags: u8,
) -> [u8; ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + 32] {
    let mut packet_buf = [0_u8; ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + 32];
    let mut tcp_header = MutableTcpPacket::new(&mut packet_buf[34..]).unwrap();

    tcp_header.set_source(src_socket.port());
    tcp_header.set_destination(dest_socket.port());
    tcp_header.set_sequence(0);
    tcp_header.set_acknowledgement(0);
    tcp_header.set_data_offset(8);
    tcp_header.set_flags(flags);
    tcp_header.set_window(64240);
    tcp_header.set_urgent_ptr(0);
    tcp_header.set_options(&[
        TcpOption::mss(1460),
        TcpOption::nop(),
        TcpOption::wscale(8),
        TcpOption::nop(),
        TcpOption::nop(),
        TcpOption::sack_perm(),
    ]);
    tcp_header.set_checksum(ipv4_checksum(
        &tcp_header.to_immutable(),
        src_socket.ip(),
        dest_socket.ip(),
    ));

    let mut ip_header = MutableIpv4Packet::new(
        &mut packet_buf[ETHERNET_HEADER_LEN..(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)],
    )
    .unwrap();
    ip_header.set_version(4);
    ip_header.set_header_length(5);
    ip_header.set_dscp(0);
    ip_header.set_ecn(0);
    ip_header.set_total_length(52);
    ip_header.set_identification(rand::random());
    ip_header.set_flags(Ipv4Flags::DontFragment);
    ip_header.set_fragment_offset(0);
    ip_header.set_ttl(128);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ip_header.set_source(src_socket.ip().clone());
    ip_header.set_destination(dest_socket.ip().clone());
    ip_header.set_checksum(checksum(&ip_header.to_immutable()));

    let mut eth_header =
        MutableEthernetPacket::new(&mut packet_buf[0..ETHERNET_HEADER_LEN]).unwrap();
    eth_header.set_destination(gateway_mac);
    eth_header.set_source(src_mac);
    eth_header.set_ethertype(EtherTypes::Ipv4);

    packet_buf
}
