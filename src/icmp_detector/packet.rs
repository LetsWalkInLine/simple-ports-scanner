use std::net::{Ipv4Addr};

use pnet::{
    packet::{
        ethernet::{EtherTypes, MutableEthernetPacket},
        icmp::{
            echo_request::{IcmpCodes, MutableEchoRequestPacket},
            IcmpTypes,
        },
        ip::IpNextHeaderProtocols,
        ipv4::{self, Ipv4Flags, MutableIpv4Packet},
        util, Packet,
    },
    util::MacAddr,
};
use rand::random;

use crate::config::{ETHERNET_HEADER_LEN, ICMP_ECHO_REQUEST_LEN, IPV4_HEADER_LEN};

pub fn build(
    src_mac: MacAddr,
    src_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    gateway_mac: MacAddr,
) -> [u8; ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + ICMP_ECHO_REQUEST_LEN] {
    let mut packet_buf = [0_u8; ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + ICMP_ECHO_REQUEST_LEN];

    let mut icmp_packet =
        MutableEchoRequestPacket::new(&mut packet_buf[ETHERNET_HEADER_LEN + IPV4_HEADER_LEN..])
            .unwrap();
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_packet.set_icmp_code(IcmpCodes::NoCode);
    icmp_packet.set_identifier(random::<u16>());
    icmp_packet.set_sequence_number(1);
    let checksum = util::checksum(icmp_packet.packet(), 1);
    icmp_packet.set_checksum(checksum);

    let mut ip_header = MutableIpv4Packet::new(
        &mut packet_buf[ETHERNET_HEADER_LEN..ETHERNET_HEADER_LEN + IPV4_HEADER_LEN],
    )
    .unwrap();
    ip_header.set_version(4);
    ip_header.set_header_length(5);
    ip_header.set_dscp(0);
    ip_header.set_ecn(0);
    ip_header.set_total_length((IPV4_HEADER_LEN + ICMP_ECHO_REQUEST_LEN) as u16);
    ip_header.set_identification(rand::random());
    ip_header.set_flags(Ipv4Flags::DontFragment);
    ip_header.set_fragment_offset(0);
    ip_header.set_ttl(128);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ip_header.set_source(src_ip);
    ip_header.set_destination(dest_ip);
    ip_header.set_checksum(ipv4::checksum(&ip_header.to_immutable()));

    let mut eth_header =
        MutableEthernetPacket::new(&mut packet_buf[0..ETHERNET_HEADER_LEN]).unwrap();
    eth_header.set_destination(gateway_mac);
    eth_header.set_source(src_mac);
    eth_header.set_ethertype(EtherTypes::Ipv4);

    packet_buf
}
