use windows_sys::Win32::{
    // https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror
    Foundation::GetLastError as get_last_error,
    System::Threading::{
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesshandlecount
        GetProcessHandleCount as get_process_handle_count,
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess
        OpenProcess as open_process,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
};

use super::*;

impl FdList {
    pub fn list(pid: Pid) -> Result<ProcStats, FshcError> {
        let mut stats = ProcStats::new(pid);

        let process_handle = unsafe { open_process(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if unsafe { get_process_handle_count(process_handle, &mut stats.file_descriptors) } == 0 {
            let code = unsafe { get_last_error() };
            Err(FshcError::from(format!("failed call code {code}")))
        } else {
            Ok(stats)
        }
    }
}
