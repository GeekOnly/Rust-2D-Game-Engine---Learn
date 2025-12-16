use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use winapi::um::processthreadsapi::GetCurrentProcess;
use std::mem;

pub fn get_memory_usage() -> usize {
    unsafe {
        let process = GetCurrentProcess();
        let mut counters: PROCESS_MEMORY_COUNTERS = mem::zeroed();
        
        if GetProcessMemoryInfo(
            process,
            &mut counters,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ) != 0
        {
            return counters.WorkingSetSize as usize;
        }
    }
    0
}
