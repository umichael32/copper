use serde_json::{json, Value};
use std::env::args;

use crate::app::error::ParamError;
use std::net::Ipv4Addr;

pub fn get_args() -> Result<Value, ParamError> {
    match args().len() {
        2 => match (args().nth(1).unwrap()).parse::<u32>() {
            Ok(p) => Ok(json!({"type" : 1 , "arg" : { "port": p }})),
            Err(_e) => Err(ParamError::new(String::from(
                "port should be a number ( err : {} )",
            ))),
        },
        4 => {
            let info: (u32, Ipv4Addr, u32) =
                match (
                    (args().nth(1).unwrap()).parse::<u32>(),
                    (args().nth(2).unwrap()).parse::<Ipv4Addr>(),
                    (args().nth(3).unwrap()).parse::<u32>(),
                ) {
                    (Ok(a), Ok(ip), Ok(b)) => (a, ip, b),
                    _ => return Err(ParamError::new(String::from(
                        "port_1 and port_2 should be digits and ip should have this format x.x.x.x",
                    ))),
                };
            Ok(json!({ "type" : 2 ,"arg" : { "port_1": info.0 ,"ip" : info.1  ,"port_2" : info.2}}))
        }
        _ => Err(ParamError::new(String::from(
            "usage : main <port> or main <port_1> <ip> <port_2> ",
        ))),
    }
}
