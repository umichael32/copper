mod app;
mod chord;

use app::parameter::{get_args, Param};
use chord::address::Address;
use chord::message::Message::Hello;
use chord::node::{listen, Node};
use std::thread::JoinHandle;

// V2
fn main() {
    if let Some(param) = get_args() {
        let t: Option<JoinHandle<()>> = match param {
            Param::Short { ip, port, id } => {
                let n: Node = Node::new(ip, port, id);
                listen(n)
            }
            Param::Long {
                ip_local,
                port_local,
                id_local,
                ip_destination,
                port_destination,
            } => {
                let n: Node = Node::new(ip_local, port_local, id_local);
                let addr_local: Address = n.get_addr();
                let t: Option<JoinHandle<()>> = listen(n);
                Address::new(ip_destination, port_destination, 0).send_message(Hello(addr_local));
                t
            }
        };
        if let Some(t) = t {
            let res = t.join();
            match res {
                Ok(..) => println!("thread gracefully stops"),
                Err(..) => println!("a problem appends in the socket's thread"),
            }
        } else {
            println!("i was not able to create the thread")
        }
    };
}
