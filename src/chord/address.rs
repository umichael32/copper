use std::net::{Ipv4Addr};

pub struct Address {
    ip: Ipv4Addr,
    port: u32,
    id: u32,
}


impl Address {
    pub fn new(ip: Ipv4Addr, port: u32, id: u32) -> Address {
        return Address { ip, port, id };
    }

    pub fn get_id(&self) -> u32 {
        return self.id;
    }

    pub fn get_ip(&self) -> Ipv4Addr {
        return self.ip;
    }

    pub fn get_port(&self) -> u32 {
        return self.port;
    }
}