use std::fs::File;
use std::io::Read;

pub fn get_memory_usage() -> usize {
    // On Android/Linux, VmRSS in /proc/self/status is a good metric
    if let Ok(mut file) = File::open("/proc/self/status") {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return kb * 1024; // Convert to bytes
                        }
                    }
                }
            }
        }
    }
    0
}
