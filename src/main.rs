mod chord;

use std::env::args;

use chord::node::Node;
use std::num::ParseIntError;
use serde_json::{Value, json};

fn get_args() -> Option<Value> {
    println!("{}", args().len());
    match args().len() {
        2 => {
            let port_str: Result<u32, ParseIntError> = (args().nth(1).unwrap()).parse::<u32>();
            match port_str {
                Ok(port) => Some(json!({"port" : port})),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            }
        }
        4 => Some(json!({})),
        _ => None,
    }
}

fn main() {
    let arg = get_args().expect("wrong param");
    println!("{}", arg.to_string());
}
