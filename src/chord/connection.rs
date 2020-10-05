use std::net::{TcpStream, Ipv4Addr};
use std::io::prelude::Write;

use crate::chord::address::Address;
use crate::chord::message::Message;

pub fn send_message(target: Address, mess: Message) -> Result<(), ()> {
    let ip: Ipv4Addr = target.get_ip();
    let mut s: TcpStream = match TcpStream::connect(format!("{}:{}", ip, target.get_port())) {
        Ok(s) => s,
        Err(_) => {
            println!("socket opening problem");
            return Err(());
        }
    };

    match s.write(mess.to_json().as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("socket send problem");
            return Err(());
        }
    }
}