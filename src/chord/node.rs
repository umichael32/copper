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
const HALF_CIRCLE: i64 = MAX_NODE / 2;

#[derive(Debug)]
pub struct Node {
    previous: (i64, Address),
    association: HashMap<i64, Address>,
    data: HashMap<i64, f64>,
    addr: Address,
    ack_vector: Vec<i64>,
    stats: (i64, i64, i64),
    exit: bool,
}

fn get_addr_from_json(json_obj: &Value, fields: &str) -> Option<Address> {
    let addr: Value = json_obj[fields].to_owned();
    let ip: Result<Ipv4Addr, AddrParseError> = addr["host"].as_str().unwrap().parse::<Ipv4Addr>();
    match (addr["id"].as_i64(), ip, addr["port"].as_i64()) {
        (Some(id), Ok(ip), Some(port)) => Some(Address::new(ip, port, id)),
        _ => None,
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
    if let Ok(sock) = TcpListener::bind(format!("{}:{}", n.addr.get_ip(), n.addr.get_port())) {
        let t_server: JoinHandle<()> = std::thread::spawn(move || {
            for stream in sock.incoming() {
                if let Ok(s) = stream {
                    n.handle_message(s);
                    println!("{:#?}", n);
                    if n.exit {
                        println!("exit");
                        break;
                    }
                } else {
                    println!("Message reception failed");
                }
            }
        });
        return Some(t_server);
    } else {
        println!(
            "Connection ip : {} on port : {} is not allowed",
            n.addr.get_ip(),
            n.addr.get_port()
        );
        None
    }
}

impl Node {
    pub fn new(ip: Ipv4Addr, port: i64, id: i64) -> Node {
        let id = id % MAX_NODE;
        let addr = Address::new(ip, port, id);
        let mut n: Node = Node {
            previous: (id, addr.clone()),
            association: HashMap::new(),
            data: HashMap::new(),
            addr: addr.clone(),
            ack_vector: Vec::new(),
            stats: (0, 0, 0),
            exit: false,
        };
        let mut idx: i64 = 1;
        while idx <= HALF_CIRCLE {
            n.association
                .insert((n.addr.get_id() + idx) % MAX_NODE, addr.clone());
            idx *= 2;
        }
        n
    }
    pub fn get_addr(&self) -> Address {
        return self.addr.clone();
    }

    fn handle_message(&mut self, stream: TcpStream) {
        if let Some(v) = read_parse(stream) {
            let s: &str = match v["cmd"].as_str() {
                Some(s) => s,
                None => return,
            };
            println!("I received : {:?}", v);
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
        } else {
            println!("Error while parsing the message")
        }
    }

    fn handle_exit(&mut self, _v: Value) {
        let previous: (i64, Address) = self.previous();
        if self.addr.get_id() != previous.0 {
            previous.1.send_message(Exit());
        }
        self.exit = true;
    }
    fn handle_ack(&mut self, args: Value) {
        if let Some(id) = args["id"].as_i64() {
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
    fn handle_answer_resp(&mut self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        if let Some(key) = args["key"].as_i64() {
            self.association.insert(key, addr);
        }
    }

    fn handle_put(&mut self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        let id: i64 = args["id"].as_i64().unwrap();
        if let Some(key) = args["key"].as_i64() {
            if let Some(n) = self.find_resp_in_table(key) {
                if let Some(v) = args["value"].as_f64() {
                    if self.addr.get_id() == n.get_id() {
                        println!("PUT : I'm updating my data");
                        self.data.insert(key, v);
                        addr.send_message(Ack(id));
                    } else {
                        println!("PUT : Send the message to the next node");
                        n.send_message(Put(addr, key, v, id));
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
        if let Some(key) = args["key"].as_i64() {
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
                let next_addr: Address = self.find_resp_in_table(key).unwrap();
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

    fn handle_get_resp(&self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        if let Some(key) = args["key"].as_i64() {
            let next_addr: Address = self.find_resp_in_table(key).unwrap();
            if self.addr.get_id() == next_addr.get_id() {
                addr.send_message(AnswerResp(key, self.addr.clone()));
            } else {
                next_addr.send_message(GetResp(addr, key));
            }
        }
    }

    fn handle_get_stat(&self, args: Value) {
        let previous: (i64, Address) = self.previous();
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        let get = args["get_amt"].as_i64().unwrap();
        let put = args["put_amt"].as_i64().unwrap();
        let mgmt = args["mgmt_amt"].as_i64().unwrap();
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
        let previous: (i64, Address) = self.previous();
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
        let resp: Option<Address> = self.find_resp_in_table(addr.get_id());
        if let Some(resp) = resp {
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
                        c.0 <= addr.get_id() || self.previous().1.get_id() > self.addr.get_id()
                    })
                    .collect();

                for x in node_data.keys() {
                    self.data.remove(x);
                }

                let (_old_previous_id, old_previous_addr) = self.previous();

                self.previous = (addr.get_id(), addr.clone());

                let data: Value = json!(node_data);

                addr.send_message(HelloOK(
                    addr.get_id(),
                    self.addr.clone(),
                    data,
                    old_previous_addr.clone(),
                ));
            }
        } else {
            println!("something went wrong while searching the responsible")
        }
    }

    fn handle_hello_ok(&mut self, args: Value) {
        let addr_previous = get_addr_from_json(&args, "address_previous");
        let addr_resp = get_addr_from_json(&args, "address_resp");
        if let Some(addr_previous) = addr_previous {
            if let Some(addr_resp) = addr_resp {
                println!("toto");
                self.previous = (addr_previous.get_id(), addr_previous);
                if let Some(data) = args["data"].as_str() {
                    let data: HashMap<i64, f64> = serde_json::from_str(data).unwrap();
                    self.data = data;
                }
                for (a, _b) in self.association.clone() {
                    addr_resp.send_message(GetResp(self.addr.clone(), a));
                }
                self.previous
                    .1
                    .send_message(UpdateTable(self.addr.clone(), -1, -1));
            } else {
                println!("something went wrong with the resp address")
            }
        } else {
            println!("something went wrong with the previous address")
        }
    }

    fn handle_hello_ko(&mut self, _args: Value) {
        self.exit = true;
    }

    fn handle_update_table(&mut self, args: Value) {
        let addr: Address = get_addr_from_json(&args, "address").unwrap();
        for (a, b) in self.association.clone() {
            if addr.get_id() > self.addr.get_id() {
                if (addr.get_id() > a && b.get_id() < addr.get_id()) && b.get_id() < a {
                    self.association.insert(a, addr.clone());
                }
            } else {
                if (addr.get_id() + MAX_NODE > a && b.get_id() < addr.get_id() + MAX_NODE)
                    && b.get_id() < a
                {
                    self.association.insert(a, addr.clone());
                }
            }
        }
    }

    pub fn previous(&self) -> (i64, Address) {
        self.previous.clone()
    }

    pub fn find_resp_in_table(&self, id: i64) -> Option<Address> {
        if (self.previous.0 == self.addr.get_id())
            || (id <= self.addr.get_id() && id > self.previous.0)
            || (id > self.previous.0
                && self.addr.get_id() > 0
                && self.previous.0 > self.addr.get_id())
            || (id > 0 && self.previous.0 > self.addr.get_id() && id <= self.addr.get_id())
        {
            return Some(self.addr.clone());
        } else {
            let exact_resp: Vec<(i64, Address)> = self
                .association
                .clone()
                .into_iter()
                .filter(|couple| couple.0 >= id && couple.1.get_id() <= id)
                .collect();
            println!("{:?}", exact_resp);
            if !exact_resp.is_empty() {
                return Some(exact_resp[0].1.clone());
            }

            let mut nearest_resp: Vec<(i64, Address)> = self
                .association
                .clone()
                .into_iter()
                .filter(|couple| couple.0 < id)
                .collect();
            nearest_resp.sort_by(|a, b| a.0.cmp(&b.0));

            println!("{:?}", nearest_resp);

            if !nearest_resp.is_empty() {
                return Some(nearest_resp[0].1.clone());
            }
        }
        None
    }
}
