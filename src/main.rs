mod app;
mod chord;

use app::parameter;
use serde_json::Value;
use std::process::exit;

fn main() {
    let arg = match parameter::get_args() {
        Ok(v) => v,
        Err(er) => {
            println!("{}", er);
            exit(1)
        }
    };
    println!("{}", arg);
}
