use crate::chord::address::Address;
use serde_json::{json, Value};

macro_rules! json_builder {
    ($cmd:expr, $arg:expr) => {
        json!({"cmd" : $cmd , "args" : $arg })
    };
}

pub enum Message {
    Ack(u64),
    Answer(u64, f64, bool),
    AnswerResp(u64, Address),
    Exit(),
    Put(Address, u64, f64, u64),
    Get(Address, u64),
    GetResp(Address, u64),
    GetStat(Address, u64, u64, u64),
    Hello(Address),
    HelloKO(u64),
    HelloOK(u64, Address, Value, Address, u64),
    Print(Address),
    UpdateTable(Address, u64, u64),
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
                "stats",
                json!({"address" : addr.to_json(), "get_amt" : get, "put_amt" : put, "mgmt_amt" : gestion  })
            ),
            Message::HelloKO(id) => json_builder!("hello_ko", json!({ "id": id })),
            Message::HelloOK(id, addr_r, data, addr_p, id_request) => json_builder!(
                "hello_ok",
                json!({"id" : id, "address_resp" : addr_r.to_json(), "data" : data , "address_previous" : addr_p.to_json(), "id_request" : id_request})
            ),
            Message::Print(addr) => json_builder!("print", json!({"address" : addr.to_json()})),
            Message::UpdateTable(addr, low_key, amount) => json_builder!(
                "update_table",
                json!({"address" : addr.to_json(), "id_lower_key" : low_key , "amount" : amount})
            ),
        }
    }
}
