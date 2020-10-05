use std::net::{Ipv4Addr, TcpStream};
use std::io::prelude::*;
use serde_json::{json, Value};

use crate::chord::message::Message;

pub struct Address {
    ip: Ipv4Addr,
    port: u32,
    id: u32,
}


impl Address {
    pub fn new(ip: Ipv4Addr, port: u32, id: u32) -> Address {
        return Address { ip, port, id };
    }

    pub fn get_id(&self) -> u32 {
        return self.id;
    }

    pub fn get_ip(&self) -> Ipv4Addr {
        return self.ip;
    }

    pub fn get_port(&self) -> u32 {
        return self.port;
    }

    fn connect(&self) -> Option<TcpStream> {
        TcpStream::connect(format!("{}:{}", self.ip, self.port)).ok()
    }

    pub fn send_message(&self, mess: Message) -> Option<usize> {
        let str_mess: String = mess.to_json().to_string();
        match self.connect() {
            Some(mut s) => s.write(str_mess.as_bytes()).ok(),
            _ => None
        }
    }

    pub fn to_json(&self) -> Value {
        json!({"id" : self.id, "host" : self.ip, "port" : self.port,})
    }
}