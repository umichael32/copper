use serde_json::{json, Value};
use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpStream};

use crate::chord::message::Message;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct Address {
    ip: Ipv4Addr,
    port: u64,
    id: u64,
}

impl Address {
    pub fn new(ip: Ipv4Addr, port: u64, id: u64) -> Address {
        return Address { ip, port, id };
    }

    pub fn get_id(&self) -> u64 {
        return self.id;
    }

    pub fn get_ip(&self) -> Ipv4Addr {
        return self.ip;
    }

    pub fn get_port(&self) -> u64 {
        return self.port;
    }

    fn connect(&self) -> Option<TcpStream> {
        TcpStream::connect(format!("{}:{}", self.ip, self.port)).ok()
    }

    pub fn send_message(&self, mess: Message) -> Option<usize> {
        let str_mess: String = mess.to_json().to_string();
        match self.connect() {
            Some(mut s) => s.write(str_mess.as_bytes()).ok(),
            _ => None,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({"id" : self.id, "host" : self.ip, "port" : self.port,})
    }
}
