use crate::chord::address::Address;
use serde_json::{json, Value};

macro_rules! json_builder {
    ($cmd:expr, $arg:expr) => {
        json!({"cmd" : $cmd , "args" : $arg })
    };
}

pub enum Message {
    Ack(i64),
    Answer(i64, f64, bool),
    AnswerResp(i64, Address),
    Exit(),
    Put(Address, i64, f64, i64),
    Get(Address, i64),
    GetResp(Address, i64),
    GetStat(Address, i64, i64, i64),
    Hello(Address),
    HelloKO(i64),
    HelloOK(i64, Address, Value, Address),
    Print(Address),
    UpdateTable(Address, i64, i64),
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
                json!({"address" : addr.to_json(), "get_amt" : get, "put_amt" : put, "mgt_amt" : gestion  })
            ),
            Message::HelloKO(id) => json_builder!("hello_ko", json!({ "id": id })),
            Message::HelloOK(id, addr_r, data, addr_p) => json_builder!(
                "hello_ok",
                json!({"id" : id, "address_resp" : addr_r.to_json(), "data" : data , "address_previous" : addr_p.to_json()})
            ),
            Message::Print(addr) => json_builder!("print", json!({"address" : addr.to_json()})),
            Message::UpdateTable(addr, low_key, amount) => json_builder!(
                "update_table",
                json!({"address" : addr.to_json(), "id_lower_key" : low_key , "amount" : amount})
            ),
        }
    }
}
