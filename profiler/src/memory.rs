use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Validates that a struct's memory layout matches expectations.
/// Useful for checking cross-platform consistency (e.g. alignment).
pub fn validate_struct_layout<T>(expected_size: usize, expected_align: usize, name: &str) {
    let size = std::mem::size_of::<T>();
    let align = std::mem::align_of::<T>();

    if size != expected_size {
        eprintln!(
            "WARNING: Struct {} size mismatch! Expected {}, got {}",
            name, expected_size, size
        );
    }
    if align != expected_align {
        eprintln!(
            "WARNING: Struct {} alignment mismatch! Expected {}, got {}",
            name, expected_align, align
        );
    }
}

/// A simple tracking allocator wrapper
pub struct TrackingAllocator<A: GlobalAlloc> {
    inner: A,
    allocated_bytes: AtomicUsize,
}

impl<A: GlobalAlloc> TrackingAllocator<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner,
            allocated_bytes: AtomicUsize::new(0),
        }
    }

    pub fn current_usage(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            self.allocated_bytes.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        self.allocated_bytes.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}
