mod app;
mod chord;
use app::client::parameter::{get_args, Param};
use chord::address::Address;
use chord::message::Message::{Exit, Get, Put};
use rand::Rng;
use serde_json::Value;
use std::io::{stdin, stdout, Read, Stdin, Write};
use std::net::TcpListener;
use std::num::ParseIntError;
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use std::thread::JoinHandle;

fn main() {
    if let Some(param) = get_args() {
        match param {
            Param::Long {
                ip,
                port,
                ip_d,
                port_d,
            } => {
                let (tx, rx) = mpsc::channel();
                let addr_d: Address = Address::new(ip_d, port_d, -1);
                let addr_l: Address = Address::new(ip, port, -1);
                let t: Option<JoinHandle<()>> = match TcpListener::bind(format!("{}:{}", ip, port))
                {
                    Ok(sock) => Some(std::thread::spawn(move || loop {
                        match rx.try_recv() {
                            Err(TryRecvError::Empty) => {
                                let stream = sock.accept();
                                if let Ok((mut s, _d)) = stream {
                                    let mut buffer: [u8; 512] = [0; 512];
                                    let u: usize = match s.read(&mut buffer) {
                                        Ok(s) => s,
                                        Err(_) => 0,
                                    };
                                    let j: Value = match serde_json::from_slice(&buffer[0..u]) {
                                        Ok(v) => v,
                                        Err(_) => serde_json::json!({}),
                                    };
                                    println!("{:?}", j);
                                }
                            }
                            Ok(_) | Err(TryRecvError::Disconnected) => break,
                        };
                    })),
                    _ => None,
                };

                if let Some(t) = t {
                    println!("there are three command :");
                    println!("get <key>");
                    println!("put <key> <value>");
                    println!("exit // to stop the client");
                    println!("stop_all // to stop the client and all the servers");
                    loop {
                        let mut rng = rand::thread_rng();
                        let mut s: String = String::new();
                        let _ = stdout().flush();
                        stdin()
                            .read_line(&mut s)
                            .expect("Did not enter a correct string");
                        if let Some('\n') = s.chars().next_back() {
                            s.pop();
                        }
                        if let Some('\r') = s.chars().next_back() {
                            s.pop();
                        }
                        println!("You typed: {}", s);

                        if s.starts_with("exit") {
                            tx.send(()).unwrap();
                            addr_l.send_message(Exit());
                            break;
                        } else if s.starts_with("stop_all") {
                            tx.send(()).unwrap();
                            addr_l.send_message(Exit());
                            addr_d.send_message(Exit());
                            break;
                        } else {
                            let cmd: Vec<&str> = s.split(" ").collect();
                            if !cmd.is_empty() {
                                match cmd[0] {
                                    "get" => {
                                        if cmd.len() == 2 {
                                            if let Ok(key) = cmd[1].parse::<i64>() {
                                                addr_d.send_message(Get(addr_l.clone(), key));
                                            } else {
                                                println!("key is not an int");
                                            }
                                        } else {
                                            println!("usage : get <key>")
                                        }
                                    }
                                    "put" => {
                                        if cmd.len() == 3 {
                                            match (cmd[1].parse::<i64>(), cmd[2].parse::<f64>()) {
                                                (Ok(key), Ok(value)) => {
                                                    let ack: i64 = rng.gen::<i64>();
                                                    addr_d.send_message(Put(
                                                        addr_l.clone(),
                                                        key,
                                                        value,
                                                        ack,
                                                    ));
                                                }
                                                (_, _) => {}
                                            }
                                        } else {
                                            println!("usage : get <key>")
                                        }
                                    }
                                    _ => println!("command not found"),
                                }
                            }
                        }
                    }

                    if let Ok(_r) = t.join() {
                        println!("thread gracefully stop");
                    } else {
                        println!("thread stopped unexpectedly ")
                    }
                }
            }
        }
    }
}
