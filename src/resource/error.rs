use std::fmt::{Display, Formatter};
use std::error::Error;

#[derive(Debug)]
pub struct ResourceNotFoundError {
    expected: String
}

impl ResourceNotFoundError {
    pub fn new(expected: String) -> Self {
        Self {
            expected
        }
    }
}

impl Display for ResourceNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Couldn't find Resource '{}' in DFS available resource list.", self.expected)
    }
}

impl Error for ResourceNotFoundError {}
