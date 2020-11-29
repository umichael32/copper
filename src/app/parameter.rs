use std::{env::args, net::Ipv4Addr};

pub enum Param {
    Short {
        ip: Ipv4Addr,
        port: i64,
        id: i64,
    },
    Long {
        ip_local: Ipv4Addr,
        port_local: i64,
        id_local: i64,
        ip_destination: Ipv4Addr,
        port_destination: i64,
    },
}

pub fn get_args() -> Option<Param> {
    let args: Vec<String> = args().collect();
    let args = args.as_slice();
    match args.len() {
        4 => match (
            args[1].parse::<Ipv4Addr>(),
            args[2].parse::<i64>(),
            args[3].parse::<i64>(),
        ) {
            (Ok(ip), Ok(port), Ok(id)) => Some(Param::Short { ip, port, id }),
            _ => None,
        },
        6 => {
            match (
                args[1].parse::<Ipv4Addr>(),
                args[2].parse::<i64>(),
                args[3].parse::<i64>(),
                args[4].parse::<Ipv4Addr>(),
                args[5].parse::<i64>(),
            ) {
                (
                    Ok(ip_local),
                    Ok(port_local),
                    Ok(id_local),
                    Ok(ip_destination),
                    Ok(port_destination),
                ) => Some(Param::Long {
                    ip_local,
                    port_local,
                    id_local,
                    ip_destination,
                    port_destination,
                }),
                _ => None,
            }
        }
        _ => None,
    }
}
