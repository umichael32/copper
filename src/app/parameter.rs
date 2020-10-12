use crate::app::error::ParamError;
use serde_json::{json, Value};
use std::{env::args, net::Ipv4Addr};

pub fn get_args() -> Result<Value, ParamError> {
    match args().len() {
        3 => match (
            (args().nth(1).unwrap()).parse::<Ipv4Addr>(),
            (args().nth(2).unwrap()).parse::<u32>(),
        ) {
            (Ok(ip), Ok(port)) => Ok(json!({"type" : 1 , "arg" : { "ip" : ip ,"port": port }})),
            _ => Err(ParamError::new(String::from(
                "use should be : copper <ip> <port>",
            ))),
        },
        5 => {
            let info: (Ipv4Addr, u32, Ipv4Addr, u32) =
                match (
                    (args().nth(1).unwrap()).parse::<Ipv4Addr>(),
                    (args().nth(2).unwrap()).parse::<u32>(),
                    (args().nth(3).unwrap()).parse::<Ipv4Addr>(),
                    (args().nth(4).unwrap()).parse::<u32>(),
                ) {
                    (Ok(ip_l), Ok(port_l), Ok(ip_d), Ok(port_d)) => (ip_l, port_l, ip_d, port_d),
                    _ => return Err(ParamError::new(String::from(
                        "port_l and port_d should be digits and ip_l and ip_d should have this format x.x.x.x",
                    ))),
                };
            Ok(
                json!({ "type" : 2 ,"arg" : { "ip_l" : info.0,"port_l": info.1 ,"ip_d" : info.2  ,"port_d" : info.3}}),
            )
        }
        _ => Err(ParamError::new(String::from(
            "usage : main <port> or main <ip_l> <port_l> <ip_d> <port_d> ",
        ))),
    }
}
