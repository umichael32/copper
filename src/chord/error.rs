use std::{error, fmt};

#[derive(Debug, Clone)]
pub struct NodeError {
    mess: String,
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while listening | {}", self.mess)
    }
}
impl error::Error for NodeError {}

impl NodeError {
    pub fn new(message: String) -> NodeError {
        return NodeError { mess: message };
    }
}
