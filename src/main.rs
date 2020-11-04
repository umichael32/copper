mod app;
mod chord;

use app::parameter;
use chord::node::Node;
use std::net::Ipv4Addr;
use std::process::exit;

fn main() {
    let args = match parameter::get_args() {
        Ok(v) => v,
        Err(er) => {
            println!("{}", er);
            exit(1)
        }
    };
    match args["type"].as_i64().unwrap() {
        1 => {
            let ip: Option<&str> = args["arg"]["ip"].as_str();
            let ip = ip.unwrap().parse::<Ipv4Addr>();
            let port = args["arg"]["port"].as_u64().unwrap();
            let mut n = Node::new(ip.unwrap(), port);
            println!("{:#?}", n);
            if let Err(e) = n.listen() {
                println!("{}", e);
            }
        }
        _ => {}
    };
}
