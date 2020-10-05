use std::fmt::{Display, Formatter, Result};
use crate::chord::address::Address;
use serde_json::{json, Value};

macro_rules! json_builder {
    ($cmd:expr; $arg:expr) => {
        json!({"cmd" : $cmd , "arg" : $arg })
    };
}


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
    pub fn to_json(&self) -> Value {
        match self {
            Message::Ack(id) =>
                json_builder!("ack"; json!({ "id" : id})),
            Message::Answer(key, value, exists) =>
                json_builder!("answer"; json!({ "key" : key, "value" : value, "val_exists" : exists})),
            Message::AnswerResp(key, addr) =>
                json_builder!("answerresp"; json!({ "key" : key, "address" : addr.to_json()})),
            Message::Exit() =>
                json_builder!("exit"; {}),
            Message::Hello(addr) =>
                json_builder!("hello"; json!({ "address" : addr.to_json()})),

            _ => { json!({}) }
        }
    }
}