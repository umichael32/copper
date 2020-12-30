use std::{env::args, net::Ipv4Addr};

pub enum Param {
    Long {
        ip: Ipv4Addr,
        port: i64,
        ip_d: Ipv4Addr,
        port_d: i64,
    },
}

pub fn get_args() -> Option<Param> {
    let args: Vec<String> = args().collect();
    let args: &[String] = args.as_slice();
    match args.len() {
        5 => match (
            args[1].parse::<Ipv4Addr>(),
            args[2].parse::<i64>(),
            args[3].parse::<Ipv4Addr>(),
            args[4].parse::<i64>(),
        ) {
            (Ok(ip_local), Ok(port_local), Ok(ip_destination), Ok(port_destination)) => {
                Some(Param::Long {
                    ip: ip_local,
                    port: port_local,
                    ip_d: ip_destination,
                    port_d: port_destination,
                })
            }
            _ => None,
        },
        _ => None,
    }
}
