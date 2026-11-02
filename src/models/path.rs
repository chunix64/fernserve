use core::fmt;

/// Max path length in bytes, including the trailing NUL.
pub const PATH_MAX: usize = 256;

#[derive(Clone, Copy)]
pub struct Path {
    buf: [u8; PATH_MAX],
    len: usize,
}

impl Path {
    pub fn new(path: &str) -> Self {
        let bytes = path.as_bytes();
        assert!(bytes.len() < PATH_MAX, "path too long");

        let mut buf = [0u8; PATH_MAX];
        buf[..bytes.len()].copy_from_slice(bytes);
        Self {
            buf,
            len: bytes.len(),
        }
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

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.as_str()
            .split('/')
            .filter(|c| !c.is_empty() && *c != ".")
    }
}

// Manual impl so Debug output matches the old derived one
// (`Path { path: "foo/bar\0" }`) instead of dumping a 256-byte array.
impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Path")
            .field("path", &self.as_str_with_nul())
            .finish()
    }
}
