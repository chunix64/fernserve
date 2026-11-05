use core::fmt;

use alloc::vec;
use alloc::vec::Vec;
use serde::{Serialize, Serializer};

use crate::models::common::PathError;

/// Max path length in bytes, including the trailing NUL.
pub const PATH_MAX: usize = 256;

#[derive(Clone, Copy)]
pub struct Path {
    buf: [u8; PATH_MAX],
    len: usize,
}

impl Path {
    pub fn new(path: &str) -> Result<Self, PathError> {
        let bytes = path.as_bytes();
        if bytes.len() > PATH_MAX {
            log::error!("[PATH] Path too long: {} > {}", bytes.len(), PATH_MAX);
            return Err(PathError::Overflow);
        }

        let mut buf = [0u8; PATH_MAX];
        buf[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            buf,
            len: bytes.len(),
        })
    }

    pub fn from_components(components: Vec<&str>) -> Result<Self, PathError> {
        let mut result = components.join("/");

        if !components.is_empty() && components[0] != "/" {
            result.insert(0, '/');
        }

        Path::new(&result)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: buf[..len] was copied from a &str in `new` and never mutated.
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }

    #[inline]
    pub fn as_str_with_nul(&self) -> &str {
        // SAFETY: same as above; NUL is valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.buf[..=self.len]) }
    }

    // pub fn raw_components(&self) -> impl Iterator<Item = &str> {
    //     self.as_str()
    //         .split('/')
    //         .filter(|c| !c.is_empty() && *c != ".")
    // }

    pub fn components(&self) -> Vec<&str> {
        let mut stack = Vec::new();

        if self.as_str() == "" || self.as_str() == "/" || self.as_str() == "/." {
            return vec!["/"];
        }

        for c in self
            .as_str()
            .split('/')
            .filter(|c| !c.is_empty() && *c != ".")
        {
            if c == ".." {
                stack.pop();
            } else {
                stack.push(c);
            }
        }

        stack
    }
}

// (`Path { path: "foo/bar\0" }`) instead of dumping [u8] array.
impl fmt::Debug for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Path")
            .field("path", &self.as_str_with_nul())
            .finish()
    }
}

impl Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
