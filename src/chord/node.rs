use crate::chord::address::Address;
use crate::chord::error::NodeError;
use crate::chord::table::Table;
use serde_json::{json, Error, Value};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::Read;
use std::net::{Ipv4Addr, TcpListener, TcpStream};

#[derive(Debug)]
pub struct Node {
    table: Table,
    data: HashMap<u32, u32>,
    addr: Address,
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u32) -> Node {
        return Node {
            table: Table::new(),
            data: HashMap::new(),
            addr: Address::new(ip, port, 1),
        };
    }

    pub fn listen(&self) -> Result<(), NodeError> {
        let sock =
            match TcpListener::bind(format!("{}:{}", self.addr.get_ip(), self.addr.get_port())) {
                Err(_e) => {
                    return Err(NodeError::new(format!(
                        "Connection ip : {} on port : {} is not allowed",
                        self.addr.get_ip(),
                        self.addr.get_port()
                    )))
                }
                Ok(s) => s,
            };
        for stream in sock.incoming() {
            let stream = match stream {
                Ok(s) => s,
                _ => return Err(NodeError::new(format!("message reception failed"))),
            };
            if let Ok(end) = self.handle_message(stream) {
                if end {
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_message(&self, mut stream: TcpStream) -> Result<bool, NodeError> {
        let mut buffer: [u8; 128] = [0; 128];
        let u = match stream.read(&mut buffer) {
            Ok(s) => s,
            Err(_) => return Err(NodeError::new(format!("message read no performed"))),
        };
        let v: Value = match serde_json::from_slice(&buffer[0..u]) {
            Ok(v) => v,
            Err(_) => return Err(NodeError::new(format!("wrong json format "))),
        };
        let s: &str = match v["cmd"].as_str() {
            Some(s) => s,
            None => return Err(NodeError::new(format!("something bad append"))),
        };
        match s {
            "exit" => Ok(true),
            _ => Ok(false),
        }
    }
}
