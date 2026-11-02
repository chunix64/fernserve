use alloc::string::{String, ToString};

pub struct Path {
    path: String,
}

impl Path {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.path.split('/').filter(|c| !c.is_empty() || *c != ".")
    }
}
