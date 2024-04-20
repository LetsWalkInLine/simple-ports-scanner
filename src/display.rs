use crate::config;
use std::{fs, io::Write, net::SocketAddrV4};

pub fn display(
    open_ports: Vec<SocketAddrV4>,
    closed_ports: Vec<SocketAddrV4>,
    filtered_ports: Vec<SocketAddrV4>,
    show_open: bool,
    show_closed: bool,
    show_filtered: bool,
) {
    let mut file = fs::File::create("output.txt").unwrap();

    if show_open {
        writeln!(file, "{:#?}", open_ports).unwrap();
    }

    if show_closed {
        writeln!(file, "{:#?}", closed_ports).unwrap();
    }
    
    if show_filtered {
        writeln!(file, "{:#?}", filtered_ports).unwrap();
    }
    
}
