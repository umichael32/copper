mod app;
mod chord;

use crate::chord::node::Node;
use app::parameter;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr};
use std::process::exit;

fn main() {
    let args = match parameter::get_args() {
        Ok(v) => v,
        Err(er) => {
            println!("{}", er);
            exit(1)
        }
    };
    match args["type"].as_i64().unwrap() {
        1 => {
            let ip = args["arg"]["ip"].as_str();
            let ip = ip.unwrap().parse::<Ipv4Addr>();
            let port = args["arg"]["port"].as_i64().unwrap() as u32;
            let mut n = Node::new(ip.unwrap(), port);

            if let Err(e) = n.listen() {
                println!("{}", e);
            }
        }
        _ => {}
    };
}
