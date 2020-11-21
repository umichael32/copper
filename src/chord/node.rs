use crate::chord::address::Address;
use crate::chord::error::NodeError;
use crate::chord::message::Message::{
    Ack, Answer, AnswerResp, Exit, Get, GetResp, GetStat, Print, Put,
};
use crate::chord::table::Table;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::net::{AddrParseError, Ipv4Addr, TcpListener, TcpStream};

#[derive(Debug)]
pub struct Node {
    table: Table,
    data: HashMap<u64, f64>,
    addr: Address,
    ack_vector: Vec<u64>,
    stats: (u64, u64, u64),
    exit: bool,
}

fn get_addr_from_json(json_obj: &Value) -> Option<Address> {
    let addr: Value = json_obj["address"].to_owned();
    let ip: Result<Ipv4Addr, AddrParseError> = addr["host"].to_string().parse::<Ipv4Addr>();
    match (addr["id"].as_u64(), ip, addr["port"].as_u64()) {
        (Some(id), Ok(ip), Some(port)) => Some(Address::new(ip, port, id)),
        _ => None,
    }
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u64) -> Node {
        let addr = Address::new(ip, port, 1);
        return Node {
            table: Table::new(1, addr.clone()),
            data: HashMap::new(),
            addr,
            ack_vector: Vec::new(),
            stats: (0, 0, 0),
            exit: false,
        };
    }

    pub fn listen(&mut self) -> Result<(), NodeError> {
        return if let Ok(sock) =
            TcpListener::bind(format!("{}:{}", self.addr.get_ip(), self.addr.get_port()))
        {
            for stream in sock.incoming() {
                let stream: TcpStream = match stream {
                    Ok(s) => s,
                    _ => return Err(NodeError::new(format!("message reception failed"))),
                };
                println!("Handle Message ");
                self.handle_message(stream);
                if self.exit {
                    break;
                }
            }
            Ok(())
        } else {
            Err(NodeError::new(format!(
                "Connection ip : {} on port : {} is not allowed",
                self.addr.get_ip(),
                self.addr.get_port()
            )))
        };
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

    fn handle_message(&mut self, stream: TcpStream) {
        if let Some(v) = Node::read_parse(stream) {
            let s: &str = match v["cmd"].as_str() {
                Some(s) => s,
                None => return,
            };
            let args = v["args"].to_owned();
            match s {
                "exit" => self.handle_exit(args),
                "ack" => self.handle_ack(args),
                "answer" => self.handle_answer(args),
                "answer_resp" => self.handle_answer_resp(args),
                "stats" => self.handle_get_stat(args),
                "print" => self.handle_print(args),
                "get" => self.handle_get(args),
                "get_resp" => self.handle_get_resp(args),
                "put" => self.handle_put(args),
                "hello" => self.handle_hello(args),
                "hello_ok" => self.handle_hello_ok(args),
                "hello_ko" => self.handle_hello_ko(args),
                "update_table" => self.handle_update_table(args),
                _ => {}
            };
        }
    }

    fn handle_exit(&mut self, _v: Value) {
        let previous: (u64, Address) = self.table.previous();
        if self.addr.get_id() != previous.0 {
            previous.1.send_message(Exit());
        }
        self.exit = true;
    }
    fn handle_ack(&mut self, args: Value) {
        if let Some(id) = args["id"].as_u64() {
            if let Some(index) = self.ack_vector.iter().position(|x| *x == id) {
                println!("ack {} accepted", id);
                self.ack_vector.remove(index);
            }
        }
    }
    fn handle_answer(&self, args: Value) {
        if let Some(key) = args["key"].as_u64() {
            if let Some(exists) = args["value_exists"].as_bool() {
                if exists {
                    if let Some(requested_value) = args["value"].as_f64() {
                        println!("the value of key {} is {}", key, requested_value);
                    };
                }
            }
        }
    }
    fn handle_answer_resp(&self, args: Value) {
        let addr: Address = get_addr_from_json(&args).unwrap();
        if let Some(key) = args["key"].as_u64() {
            println!("key {}, Responsible {:?}", key, addr);
        }
    }

    fn handle_put(&mut self, args: Value) {
        let addr: Address = get_addr_from_json(&args).unwrap();
        let id: u64 = args["id"].as_u64().unwrap();
        if let Some(key) = args["key"].as_u64() {
            if let Some(n) = self.table.get_node(key) {
                if let Some(v) = args["value"].as_f64() {
                    if self.addr.get_id() == n.0 {
                        self.data.insert(key, v);
                        addr.send_message(Ack(id));
                    } else {
                        n.1.send_message(Put(addr, key, v, id));
                    }
                }
            }
        }
    }

    fn handle_get(&self, args: Value) {
        let addr: Address = get_addr_from_json(&args).unwrap();
        // Get request's key
        if let Some(key) = args["key"].as_u64() {
            // Try to see if the node already has the key
            if let Some(v) = self.data.get(&key) {
                // yes
                if self.addr == addr {
                    // If i'm the one who ask the key then i print it
                    println!("{}", v);
                } else {
                    // else i send the response to the node who requested it
                    addr.send_message(Answer(key, *v, true));
                }
            } else {
                // if I do not own the key
                // find which table has it
                let (_0, _1): (u64, Address) = self.table.get_node(key).unwrap();
                if self.addr.get_id() == _0 {
                    // if i'm the one who normally has it then send an error
                    addr.send_message(Answer(key, 0.0, false));
                } else {
                    // else send the request to the next node
                    _1.send_message(Get(addr, key));
                }
            }
        }
    }

    fn handle_get_resp(&self, args: Value) {
        let addr: Address = get_addr_from_json(&args).unwrap();
        if let Some(key) = args["key"].as_u64() {
            let (_0, _1): (u64, Address) = self.table.get_node(key).unwrap();
            if self.addr.get_id() == _0 {
                addr.send_message(AnswerResp(self.addr.get_id(), self.addr.clone()));
            } else {
                _1.send_message(GetResp(addr, key));
            }
        }
    }

    fn handle_get_stat(&self, args: Value) {
        let previous: (u64, Address) = self.table.previous();
        let addr: Address = get_addr_from_json(&args).unwrap();
        let get = args["get_amt"].as_u64().unwrap();
        let put = args["put_amt"].as_u64().unwrap();
        let mgmt = args["mgmt_amt"].as_u64().unwrap();
        if self.addr.get_id() != previous.0 {
            previous.1.send_message(GetStat(
                addr,
                get + self.stats.0,
                put + self.stats.1,
                mgmt + self.stats.2,
            ));
        } else {
            println!("get {}, put {}, management {}", get, put, mgmt);
        }
    }

    fn handle_print(&self, args: Value) {
        let previous: (u64, Address) = self.table.previous();
        let addr: Address = get_addr_from_json(&args).unwrap();
        if self.addr.get_id() != previous.0 {
            println!(
                "get {}, put {}, management {}",
                self.stats.0, self.stats.1, self.stats.2
            );
            previous.1.send_message(Print(addr));
        }
    }

    fn handle_hello(&self, _args: Value) {}

    fn handle_hello_ok(&self, _args: Value) {}

    fn handle_hello_ko(&self, _args: Value) {}

    fn handle_update_table(&self, _args: Value) {}
}
