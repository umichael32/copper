mod chord;

use chord::connection::send_message;
use chord::address::Address;
use chord::message::Message;
use std::net::Ipv4Addr;

fn main() {
    let addr_request: Address = Address::new(Ipv4Addr::new(127, 0, 0, 1), 8887, 1);
    let addr_dest: Address = Address::new(Ipv4Addr::new(127, 0, 0, 1), 8888, 1);
    let mess: Message = Message::Hello(addr_request);
    match send_message(addr_dest, mess) {
        Ok(_) => println!("OK"),
        Err(_) => println!("Err"),
    };
}
