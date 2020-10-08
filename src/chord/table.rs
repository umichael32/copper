use std::collections::HashMap;
use crate::chord::address::Address;

pub struct Table {
    association: HashMap<u32, Address>,
}

impl Table {
    pub fn new() -> Table {
        return Table { association: HashMap::new() };
    }
}
