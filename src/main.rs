mod app;
mod chord;

use crate::chord::address::Address;
use crate::chord::message::Message::Hello;
use crate::chord::node::{listen, Node};
use app::parameter;
use serde_json::Value;
use std::net::{AddrParseError, Ipv4Addr};
use std::process::exit;

fn main() {
    let args: Value = match parameter::get_args() {
        Ok(v) => v,
        Err(er) => {
            println!("{}", er);
            exit(1)
        }
    };
    let t = match args["type"].as_i64().unwrap() {
        2 => {
            let ip: Option<&str> = args["arg"]["ip_l"].as_str();
            let ip: Result<Ipv4Addr, AddrParseError> = ip.unwrap().parse::<Ipv4Addr>();
            let port: i64 = args["arg"]["port_l"].as_i64().unwrap();
            let id: i64 = args["arg"]["id"].as_i64().unwrap();
            let n: Node = Node::new(ip.unwrap(), port, id);
            let addr_s = n.get_addr();
            println!("{:?}", n);
            let r = listen(n);

            let ip: Option<&str> = args["arg"]["ip_d"].as_str();
            let ip: Result<Ipv4Addr, AddrParseError> = ip.unwrap().parse::<Ipv4Addr>();
            let port: i64 = args["arg"]["port_d"].as_i64().unwrap();
            let addr_d = Address::new(ip.unwrap(), port, 0);
            addr_d.send_message(Hello(addr_s));

            r
        }
        1 => {
            let ip: Option<&str> = args["arg"]["ip"].as_str();
            let ip: Result<Ipv4Addr, AddrParseError> = ip.unwrap().parse::<Ipv4Addr>();
            let port: i64 = args["arg"]["port"].as_i64().unwrap();
            let id: i64 = args["arg"]["id"].as_i64().unwrap();
            let n: Node = Node::new(ip.unwrap(), port, id);
            println!("{:?}", n);
            listen(n)
        }
        _ => None,
    };
    if let Some(t) = t {
        let _r = t.join();
    }
}
