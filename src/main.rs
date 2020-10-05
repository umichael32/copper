mod chord;

use chord::address::Address;
use chord::message::Message;
use std::net::Ipv4Addr;

fn main() {
    let addr_request: Address = Address::new(Ipv4Addr::new(127, 0, 0, 1), 8887, 1);
    let addr_dest: Address = Address::new(Ipv4Addr::new(127, 0, 0, 1), 8888, 1);
    let mess: Message = Message::Exit();

    match addr_dest.send_message(mess) {
        Some(size) => println!("Ok i wrote {} charactere ", size),
        None => println!("A problem happen"),
    };
}
