use crate::chord::address::Address;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Table {
    previous: (u64, Address),
    association: HashMap<u64, Address>,
}

impl Table {
    pub fn new(id: u64, addr: Address) -> Table {
        let mut table = Table {
            previous: (id, addr.clone()),
            association: HashMap::new(),
        };
        table.association.insert(id, addr);
        return table;
    }

    pub fn previous(&self) -> (u64, Address) {
        self.previous.clone()
    }
}
