use std::fmt::{Display, Formatter, Result};
use crate::chord::address::Address;
use serde_json::json;

pub enum Message {
    Ack(u32),
    Answer(u32, u32, bool),
    AnswerResp(u32, Address),
    Exit(),
    Put(Address, u32, u32, u32),
    Get(Address, u32),
    GetResp(Address, u32),
    GetStat(u32, u32, u32, u32),
    Hello(Address),
    HelloKO(u32),
    HelloOK(u32, Address, String, Address, u32),
    Print(u32),
    UpdateTable(Address, i32, i32),
}

impl Message {
    pub fn to_json(&self) -> String {
        let j = match self {
            Message::Hello(addr) => json!({ "idNode" : addr.get_id(), "IP" : addr.get_ip(), "port" : addr.get_port(),}),
            _ => { json!({}) }
        };
        j.to_string()
    }
}