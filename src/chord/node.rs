use crate::chord::table::Table;


pub struct Node {
    table: Table,
}

impl Node {
    pub fn new() -> Node {
        return Node { table: Table::new() };
    }
    pub fn listen(&self) {}
} 