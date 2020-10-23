use crate::chord::address::Address;
use serde_json::{json, Value};

macro_rules! json_builder {
    ($cmd:expr, $arg:expr) => {
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
    GetStat(Address, u32, u32, u32),
    Hello(Address),
    HelloKO(u32),
    HelloOK(u32, Address, Value, Address, u32),
    Print(Address),
    UpdateTable(Address, i32, i32),
}

impl Message {
    pub fn to_json(&self) -> Value {
        match self {
            Message::Ack(id) => json_builder!("ack", json!({ "id": id })),
            Message::Answer(key, value, exists) => json_builder!(
                "answer",
                json!({ "key" : key, "value" : value, "val_exists" : exists})
            ),
            Message::AnswerResp(key, addr) => json_builder!(
                "answer_resp",
                json!({ "key" : key, "address" : addr.to_json()})
            ),
            Message::Exit() => json_builder!("exit", {}),
            Message::Hello(addr) => json_builder!("hello", json!({ "address" : addr.to_json()})),
            Message::Put(addr, key, value, id) => json_builder!(
                "put",
                json!({"address" : addr.to_json() ,"key" : key, "value" : value , "id" : id})
            ),
            Message::Get(addr, key) => {
                json_builder!("get", json!({"address" : addr.to_json(), "key" : key}))
            }
            Message::GetResp(addr, key) => {
                json_builder!("get_resp", json!({"address" : addr.to_json(), "key" : key}))
            }
            Message::GetStat(addr, get, put, gestion) => json_builder!(
                "get_stat",
                json!({"address" : addr.to_json(), "get" : get, "put" : put, "gestion" : gestion  })
            ),
            Message::HelloKO(id) => json_builder!("hello_ko", json!({ "id": id })),
            Message::HelloOK(id, addr_r, data, addr_p, id_request) => json_builder!(
                "hello_ok",
                json!({"id_request" : id, "address_resp" : addr_r.to_json(), "data" : data , "address_previous" : addr_p.to_json(), "id_request" : id_request})
            ),
            Message::Print(addr) => json_builder!("print", json!({"address" : addr.to_json()})),
            Message::UpdateTable(addr, low_key, amount) => json_builder!(
                "update_table",
                json!({"address" : addr.to_json(), "id_lower_key" : low_key , "amount" : amount})
            ),
        }
    }
}
