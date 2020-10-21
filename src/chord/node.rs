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
    data: HashMap<u32, f64>,
    addr: Address,
    ack_vector: Box<Vec<u32>>,
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u32) -> Node {
        return Node {
            table: Table::new(),
            data: HashMap::new(),
            addr: Address::new(ip, port, 1),
            ack_vector: Box::new(Vec::new()),
        };
    }

    pub fn listen(&mut self) -> Result<(), NodeError> {
        return if let Ok(sock) = TcpListener::bind(format!("{}:{}", self.addr.get_ip(), self.addr.get_port())) {
            for stream in sock.incoming() {
                let stream: TcpStream = match stream {
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
        } else {
            Err(NodeError::new(format!(¶"Connection ip : {} on port : {} is not allowed",self.addr.get_ip(),self.addr.get_port())))
        }
    }

    fn read_parse(mut stream: TcpStream) -> Option<Value> {
        let mut buffer: [u8; 128] = [0; 128];
        let u: usize = match stream.read(&mut buffer) {
            Ok(s) => s,
            Err(_) => return None,
        };
        match serde_json::from_slice(&buffer[0..u]) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    fn handle_message(&mut self, mut stream: TcpStream) -> Result<bool, NodeError> {
        if let Some(v) = Node::read_parse(stream) {
            let s: &str = match v["cmd"].as_str() {
                Some(s) => s,
                None => return Err(NodeError::new(format!("something bad append"))),
            };
            let b = match s {
                "exit" => true,
                "ack" => self.handle_ack(v),
                "answer" => self.handle_answer(v),
                "answer_resp" => self.handle_answer_resp(v),
                _ => false,
            };
            Ok(b)
        } else {
            Err(NodeError::new(format!(
                "error while reading the incoming message ( read parse ) "
            )))
        }
    }
    fn handle_ack(&mut self, v: Value) -> bool {
        let args: Value = v["args"].to_owned();
        if let Some(id) = args["id"].as_u64() {
            match self.ack_vector.iter().position(|x| *x == (id as u32)) {
                Some(index) => {
                    println!("ack {} accepted", id);
                    self.ack_vector.remove(index);
                }
                None => println!("I don't have this ack"),
            }
        }
        return false;
    }
    fn handle_answer(&self, v: Value) -> bool {
        let args: Value = v["args"].to_owned();
        let key: u32 = args["key"].as_u64().unwrap() as u32;
        if let Some(exists) = args["value_exists"].as_bool() {
            if exists {
                match args["value"].as_f64() {
                    Some(requested_value) => {
                        println!("the value of key {} is {}", key, requested_value)
                    }
                    None => println!("the value is not f64 able"),
                };
            } else {
                println!("the value don't exist\n");
            }
        }
        false
    }
    fn handle_answer_resp(&self, v: Value) -> bool {
        let args: Value = v["args"].to_owned();
        false
    }
}
