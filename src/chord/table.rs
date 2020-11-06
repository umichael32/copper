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

    pub fn get_node(&self, id: u64) -> Option<(u64, Address)> {
        let candidate: Vec<(u64, Address)> = self
            .association
            .clone()
            .into_iter()
            .filter(|e| e.0 < id)
            .collect();
        println!("{:#?}", candidate);
        let r: Option<(u64, Address)> = candidate.get(0).cloned();
        return r;
    }
}
