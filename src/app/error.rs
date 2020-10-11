use std::{error, fmt, net};

#[derive(Debug, Clone)]
pub struct ParamError {
    mess: String,
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid parameter | {}", self.mess)
    }
}
impl error::Error for ParamError {}

impl ParamError {
    pub fn new(message: String) -> ParamError {
        return ParamError { mess: message };
    }
}
