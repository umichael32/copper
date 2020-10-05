use std::collections::HashMap;
use crate::chord::address::Address;

struct Table {
    association: HashMap<u32, Address>,
}