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
            println!("message incoming");
            let stream = match stream {
                Ok(s) => s,
                _ => return Err(NodeError::new(format!("message reception failed"))),
            };
            self.handle_message(stream);
        }
        Ok(())
    }

    fn handle_message(&self, mut stream: TcpStream) {
        let mut buffer = [0; 128];
        stream.read(&mut buffer).unwrap();
    }
}
