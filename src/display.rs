use crate::config::get_port_name;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    net::{Ipv4Addr, SocketAddrV4},
};

struct TargetStates {
    open: Vec<(u16, &'static str)>,
    closed: Vec<(u16, &'static str)>,
    filtered: Vec<(u16, &'static str)>,
}

impl TargetStates {
    fn new() -> Self {
        Self {
            open: Vec::new(),
            closed: Vec::new(),
            filtered: Vec::new(),
        }
    }
}

pub fn display(
    open_ports: Vec<SocketAddrV4>,
    closed_ports: Vec<SocketAddrV4>,
    filtered_ports: Vec<SocketAddrV4>,
    show_open: bool,
    show_closed: bool,
    show_filtered: bool,
) {
    let tree = get_info_tree(&open_ports, &closed_ports, &filtered_ports);

    let mut file = fs::File::create("example/output.toml").unwrap();

    writeln!(file, "[summary]").unwrap();
    writeln!(
        file,
        "total = {}",
        open_ports.len() + closed_ports.len() + filtered_ports.len()
    )
    .unwrap();
    writeln!(file, "open = {}", open_ports.len()).unwrap();
    writeln!(file, "closed = {}", closed_ports.len()).unwrap();
    writeln!(file, "filtered = {}", filtered_ports.len()).unwrap();
    writeln!(file).unwrap();

    writeln!(file, "[[target]]").unwrap();

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

fn get_info_tree(
    open_ports: &[SocketAddrV4],
    closed_ports: &[SocketAddrV4],
    filtered_ports: &[SocketAddrV4],
) -> BTreeMap<Ipv4Addr, TargetStates> {
    let mut tree = BTreeMap::<Ipv4Addr, TargetStates>::new();

    open_ports.iter().for_each(|x| {
        tree.entry(*x.ip())
            .and_modify(|v| {
                v.open
                    .push((x.port(), get_port_name(x.port()).unwrap_or("Known")))
            })
            .or_insert(TargetStates::new());
    });

    closed_ports.iter().for_each(|x| {
        tree.entry(*x.ip())
            .and_modify(|v| {
                v.closed
                    .push((x.port(), get_port_name(x.port()).unwrap_or("Known")))
            })
            .or_insert(TargetStates::new());
    });

    filtered_ports.iter().for_each(|x| {
        tree.entry(*x.ip())
            .and_modify(|v| {
                v.filtered
                    .push((x.port(), get_port_name(x.port()).unwrap_or("Known")))
            })
            .or_insert(TargetStates::new());
    });

    tree
}