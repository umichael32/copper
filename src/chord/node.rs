use crate::chord::address::Address;
use crate::chord::message::Message::{
    Ack, Answer, AnswerResp, Exit, Get, GetResp, GetStat, Hello, HelloKO, HelloOK, Print, Put,
    UpdateTable,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::net::{AddrParseError, Ipv4Addr, TcpListener, TcpStream};
use std::thread::JoinHandle;

const MAX_NODE: i64 = 32;

#[derive(Debug)]
pub struct Node {
    previous: Address,
    next: Address,
    data: HashMap<i64, f64>,
    addr: Address,
    ack_vector: Vec<i64>,
    put: i64,
    get: i64,
    mgt: i64,
    exit: bool,
}

fn get_addr_from_json(json_obj: &Value, fields: &str) -> Option<Address> {
    let addr: Value = json_obj[fields].to_owned();
    if let Some(ip_str) = addr["ip"].as_str() {
        let ip: Result<Ipv4Addr, AddrParseError> = ip_str.parse::<Ipv4Addr>();
        match (addr["id"].as_i64(), ip, addr["port"].as_i64()) {
            (Some(id), Ok(ip), Some(port)) => Some(Address::new(ip, port, id)),
            _ => None,
        }
    } else {
        None
    }
}

fn read_parse(mut stream: TcpStream) -> Option<Value> {
    let mut buffer: [u8; 512] = [0; 512];
    let u: usize = match stream.read(&mut buffer) {
        Ok(s) => s,
        Err(_) => return None,
    };
    match serde_json::from_slice(&buffer[0..u]) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

pub fn listen(mut n: Node) -> Option<JoinHandle<()>> {
    match TcpListener::bind(format!("{}:{}", n.addr.get_ip(), n.addr.get_port())) {
        Ok(sock) => Some(std::thread::spawn(move || {
            for stream in sock.incoming() {
                if let Ok(s) = stream {
                    n.handle_message(s);
                    println!("{:?}", n);
                    if n.exit {
                        println!("exit");
                        break;
                    }
                } else {
                    println!("Message reception failed");
                }
            }
        })),
        _ => None,
    }
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: i64, id: i64) -> Node {
        let id: i64 = id % MAX_NODE;
        let addr: Address = Address::new(ip, port, id);
        Node {
            previous: addr.clone(),
            next: addr.clone(),
            data: HashMap::new(),
            addr: addr.clone(),
            ack_vector: Vec::new(),
            get: 0,
            put: 0,
            mgt: 0,
            exit: false,
        }
    }
    pub fn get_addr(&self) -> Address {
        return self.addr.clone();
    }

    fn handle_message(&mut self, stream: TcpStream) {
        if let Some(v) = read_parse(stream) {
            if let Some(s) = v["cmf"].as_str() {
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
    }

    fn handle_exit(&mut self, _v: Value) {
        if self.addr.get_id() != self.previous.get_id() {
            self.previous.send_message(Exit());
        }
        self.exit = true;
    }
    fn handle_ack(&mut self, args: Value) {
        self.mgt += 1;
        if let Some(id) = args["id"].as_i64() {
            if let Some(index) = self.ack_vector.iter().position(|x| *x == id) {
                println!("ack {} accepted", id);
                self.ack_vector.remove(index);
            }
        }
    }
    fn handle_answer(&self, args: Value) {
        if let Some(key) = args["key"].as_i64() {
            if let Some(exists) = args["value_exists"].as_bool() {
                if exists {
                    if let Some(requested_value) = args["value"].as_f64() {
                        println!("the value of key {} is {}", key, requested_value);
                    };
                }
            }
        }
    }
    fn handle_answer_resp(&mut self, args: Value) {
        if let Some(addr) = get_addr_from_json(&args, "address") {
            if let Some(key) = args["key"].as_i64() {
                println!("key {}, address {:?}", key, addr);
            }
        }
    }

    fn handle_put(&mut self, args: Value) {
        self.put += 1;
        if let Some(addr) = get_addr_from_json(&args, "address") {
            match (
                args["id"].as_i64(),
                args["key"].as_i64(),
                args["value"].as_f64(),
            ) {
                (Some(id), Some(key), Some(v)) => {
                    if let Some(n) = self.find_resp_in_table(key) {
                        if self.addr.get_id() == n.get_id() {
                            println!("PUT : I'm updating my data");
                            self.data.insert(key, v);
                            addr.send_message(Ack(id));
                        } else {
                            println!("PUT : Send the message to the next node");
                            n.send_message(Put(addr, key, v, id));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_get(&mut self, args: Value) {
        if let Some(addr) = get_addr_from_json(&args, "address") {
            self.get += 1;
            // get request's key
            if let Some(key) = args["key"].as_i64() {
                // try to see if the node already has the key
                if let Some(v) = self.data.get(&key) {
                    // yes
                    if self.addr == addr {
                        // if i'm the one who ask the key then i print it
                        println!("{}", v);
                    } else {
                        // else i send the response to the node who requested it
                        addr.send_message(Answer(key, *v, true));
                    }
                } else {
                    // if i do not own the key
                    // find which table has it
                    if let Some(next_addr) = self.find_resp_in_table(key) {
                        if self.addr.get_id() == next_addr.get_id() {
                            // if i'm the one who normally has it then send an error
                            addr.send_message(Answer(key, 0.0, false));
                        } else {
                            // else send the request to the next node
                            next_addr.send_message(Get(addr, key));
                        }
                    }
                }
            }
        }
    }

    fn handle_get_resp(&self, args: Value) {
        if let Some(addr) = get_addr_from_json(&args, "address") {
            if let Some(key) = args["key"].as_i64() {
                if let Some(next_addr) = self.find_resp_in_table(key) {
                    if self.addr.get_id() == next_addr.get_id() {
                        addr.send_message(AnswerResp(key, self.addr.clone()));
                    } else {
                        next_addr.send_message(GetResp(addr, key));
                    }
                }
            }
        }
    }

    fn handle_get_stat(&self, args: Value) {
        if let Some(addr) = get_addr_from_json(&args, "address") {
            match (
                args["get_amt"].as_i64(),
                args["put_amt"].as_i64(),
                args["mgt_amt"].as_i64(),
            ) {
                (Some(get), Some(put), Some(mgt))
                    if self.addr.get_id() != self.previous.get_id() =>
                {
                    self.previous.send_message(GetStat(
                        addr,
                        get + self.get,
                        put + self.put,
                        mgt + self.mgt,
                    ));
                }
                (Some(get), Some(put), Some(mgt)) => {
                    println!("get {}, put {}, management {}", get, put, mgt);
                }
                _ => {}
            };
        }
    }

    fn handle_print(&self, args: Value) {
        if let Some(addr) = get_addr_from_json(&args, "address") {
            if self.addr.get_id() != self.previous.get_id() {
                println!(
                    "get {}, put {}, management {}",
                    self.get, self.put, self.mgt
                );
                self.previous.send_message(Print(addr));
            }
        }
    }

    fn handle_hello(&mut self, args: Value) {
        if let Some(addr) = get_addr_from_json(&args, "address") {
            if let Some(resp) = self.find_resp_in_table(addr.get_id()) {
                println!("{:?}", resp);
                if resp.get_id() != self.addr.get_id() {
                    resp.send_message(Hello(addr));
                } else if self.addr.get_id() == addr.get_id() {
                    addr.send_message(HelloKO(addr.get_id()));
                } else {
                    let node_data: HashMap<i64, f64> = self
                        .data
                        .clone()
                        .into_iter()
                        .filter(|c| {
                            c.0 <= addr.get_id() || self.previous.get_id() > self.addr.get_id()
                        })
                        .collect();

                    for x in node_data.keys() {
                        self.data.remove(x);
                    }

                    let old_previous: Address = self.previous.clone();

                    self.previous = addr.clone();

                    let data: Value = json!(node_data);

                    addr.send_message(HelloOK(
                        addr.get_id(),
                        self.addr.clone(),
                        data,
                        old_previous.clone(),
                    ));
                }
            }
        }
    }

    fn handle_hello_ok(&mut self, args: Value) {
        if let Some(addr_previous) = get_addr_from_json(&args, "address_previous") {
            if let Some(addr_resp) = get_addr_from_json(&args, "address_resp") {
                self.previous = addr_previous;
                if let Some(data) = args["data"].as_str() {
                    self.data = serde_json::from_str(data).unwrap();
                }
                self.next = addr_resp;
                self.previous
                    .send_message(UpdateTable(self.addr.clone(), -1, -1));
            }
        }
    }

    fn handle_hello_ko(&mut self, _args: Value) {
        self.exit = true;
    }

    fn handle_update_table(&mut self, args: Value) {
        self.mgt += 1;
        if let Some(addr) = get_addr_from_json(&args, "address") {
            self.next = addr;
        }
    }
    fn is_between(id: i64, lower: i64, upper: i64) -> bool {
        return (lower == upper)
            || (id <= upper && id > lower)
            || (id > lower && upper >= 0 && lower > upper)
            || (id >= 0 && lower > upper && id <= upper)
            || (id == upper);
    }
    fn is_mine(&self, id: i64) -> bool {
        let previous_id: i64 = self.previous.get_id();
        let my_id: i64 = self.addr.get_id();
        return Node::is_between(id, previous_id, my_id);
    }

    pub fn find_resp_in_table(&self, id: i64) -> Option<Address> {
        let id: i64 = id % MAX_NODE;
        return if self.is_mine(id) {
            Some(self.addr.clone())
        } else {
            Some(self.next.clone())
        };
    }
}
