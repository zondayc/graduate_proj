use crate::alloc::string::ToString;
use alloc::string::String;

/// The struct contains the file path.
pub struct Path(String);

impl Path {
    /// Create a new path from a string.
    #[inline]
    pub fn new(path: &str) -> Self {
        Self(path.to_string())
    }

    pub fn as_path(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
