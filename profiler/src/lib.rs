pub mod memory;
pub mod platform;
pub mod macros;
pub mod gpu;

#[cfg(feature = "enable_profiling")]
pub struct ScopeTimer<'a> {
    name: &'a str,
    start: std::time::Instant,
}

#[cfg(feature = "enable_profiling")]
impl<'a> ScopeTimer<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

#[cfg(feature = "enable_profiling")]
impl<'a> Drop for ScopeTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        // In a real engine, you'd record this to a global profiler instance
        // log::trace!("Profile [{}]: {:?}", self.name, duration);
    }
}

/// Simple micro-benchmark helper for ECS logic
pub fn micro_bench<F>(name: &str, iterations: u32, mut func: F) 
where F: FnMut() 
{
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        func();
    }
    let duration = start.elapsed();
    let avg = duration.as_secs_f64() * 1000.0 / iterations as f64;
    println!("BENCH [{}]: Total = {:?}, Avg = {:.4}ms/iter", name, duration, avg);
}
