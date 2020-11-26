use crate::chord::address::Address;
use crate::chord::message::Message::{
    Ack, Answer, AnswerResp, Exit, Get, GetResp, GetStat, Hello, HelloKO, HelloOK, Print, Put,
    UpdateTable,
};
use crate::chord::table::Table;
use serde_json::{json, Value};
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

fn get_addr_from_json(json_obj: &Value, fields: &str) -> Option<Address> {
    let addr: Value = json_obj[fields].to_owned();
    let ip: Result<Ipv4Addr, AddrParseError> = addr["host"].as_str().unwrap().parse::<Ipv4Addr>();
    match (addr["id"].as_u64(), ip, addr["port"].as_u64()) {
        (Some(id), Ok(ip), Some(port)) => Some(Address::new(ip, port, id)),
        _ => None,
    }
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: u64, id: u64) -> Node {
        let addr = Address::new(ip, port, id);
        return Node {
            table: Table::new(id, addr.clone()),
            data: HashMap::new(),
            addr,
            ack_vector: Vec::new(),
            stats: (0, 0, 0),
            exit: false,
        };
    }
    pub fn get_addr(&self) -> Address {
        return self.addr.clone();
    }

    pub fn listen(&mut self) {
        if let Ok(sock) =
            TcpListener::bind(format!("{}:{}", self.addr.get_ip(), self.addr.get_port()))
        {
            for stream in sock.incoming() {
                if let Ok(s) = stream {
                    self.handle_message(s);
                    println!("{:?}", self);
                    if self.exit {
                        break;
                    }
                } else {
                    println!("Message reception failed");
                }
            }
        } else {
            println!(
                "Connection ip : {} on port : {} is not allowed",
                self.addr.get_ip(),
                self.addr.get_port()
            )
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

    fn handle_message(&mut self, stream: TcpStream) {
        if let Some(v) = Node::read_parse(stream) {
            let s: &str = match v["cmd"].as_str() {
                Some(s) => s,
                None => return,
            };
            println!("I received : {}", s);
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
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        if let Some(key) = args["key"].as_u64() {
            println!("key {}, Responsible {:?}", key, addr);
        }
    }

    fn handle_put(&mut self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        let id: u64 = args["id"].as_u64().unwrap();
        if let Some(key) = args["key"].as_u64() {
            if let Some(n) = self.table.get_node(key) {
                if let Some(v) = args["value"].as_f64() {
                    if self.addr.get_id() == n.0 {
                        println!("PUT : I'm updating my data");
                        self.data.insert(key, v);
                        addr.send_message(Ack(id));
                    } else {
                        println!("PUT : Send the message to the next node");
                        n.1.send_message(Put(addr, key, v, id));
                    }
                } else {
                    println!("PUT : Value problem");
                }
            } else {
                println!("PUT : Node problem");
            }
        } else {
            println!("PUT : Key problem");
        }
    }

    fn handle_get(&self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
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
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
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
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
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
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        if self.addr.get_id() != previous.0 {
            println!(
                "get {}, put {}, management {}",
                self.stats.0, self.stats.1, self.stats.2
            );
            previous.1.send_message(Print(addr));
        }
    }

    fn handle_hello(&mut self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        let resp: Option<(u64, Address)> = self.table.get_node(addr.get_id());
        if let Some(resp) = resp {
            if resp.0 != self.addr.get_id() {
                resp.1.send_message(Hello(addr));
            } else if self.addr.get_id() == addr.get_id() {
                addr.send_message(HelloKO(addr.get_id()));
            } else {
                let node_data: HashMap<u64, f64> = self
                    .data
                    .clone()
                    .into_iter()
                    .filter(|c| {
                        c.0 <= addr.get_id()
                            || self.table.previous().1.get_id() > self.addr.get_id()
                    })
                    .collect();

                for x in node_data.keys() {
                    self.data.remove(x);
                }

                let (_old_previous_id, old_previous_addr) = self.table.previous();

                self.table.previous = (addr.get_id(), addr.clone());

                let data: Value = json!(node_data);

                addr.send_message(HelloOK(
                    addr.get_id(),
                    self.addr.clone(),
                    data,
                    self.table.previous().1.clone(),
                ));
                old_previous_addr.send_message(UpdateTable(addr.clone(), -1, -1));
            }
        }
    }

    fn handle_hello_ok(&mut self, args: Value) {
        let addr_previous = get_addr_from_json(&args, "address_previous");
        let addr_resp = get_addr_from_json(&args, "address_resp");
        if let Some(addr_previous) = addr_previous {
            if let Some(addr_resp) = addr_resp {
                self.table.previous = (addr_previous.get_id(), addr_previous);
                self.table
                    .association
                    .insert(self.addr.get_id() + 1, addr_resp);
                let data: HashMap<u64, f64> =
                    serde_json::from_str(args["data"].as_str().unwrap()).unwrap();
                self.data = data;
            }
        }
    }

    fn handle_hello_ko(&mut self, _args: Value) {
        self.exit = true;
    }

    fn handle_update_table(&self, _args: Value) {}
}
