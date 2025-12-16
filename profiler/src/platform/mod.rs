#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use self::android::*;

#[cfg(not(any(target_os = "windows", target_os = "android")))]
pub fn get_memory_usage() -> usize {
    0 // Not implemented for other OS
}
