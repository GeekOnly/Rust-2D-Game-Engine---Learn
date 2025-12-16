use std::sync::atomic::{AtomicUsize, Ordering};

pub static DRAW_CALLS: AtomicUsize = AtomicUsize::new(0);
pub static TRIANGLE_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn reset_frame_counters() {
    DRAW_CALLS.store(0, Ordering::Relaxed);
    TRIANGLE_COUNT.store(0, Ordering::Relaxed);
}

pub fn add_draw_call(tris: usize) {
    DRAW_CALLS.fetch_add(1, Ordering::Relaxed);
    TRIANGLE_COUNT.fetch_add(tris, Ordering::Relaxed);
}

pub fn get_stats() -> (usize, usize) {
    (
        DRAW_CALLS.load(Ordering::Relaxed),
        TRIANGLE_COUNT.load(Ordering::Relaxed),
    )
}
