use alloc::boxed::Box;
use core::ptr::NonNull;

pub struct Heap<T>(NonNull<T>);

impl<T> Heap<T> {
    pub fn new(value: T) -> Self {
        Self(NonNull::from(Box::leak(Box::new(value))))
    }
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> Drop for Heap<T> {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.0.as_ptr()) });
    }
}

unsafe impl<T: Send> Send for Heap<T> {}
