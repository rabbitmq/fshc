use super::*;

use std::ffi::c_void;
use windows_sys::Win32::{
    Foundation::{STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS},
    System::WindowsProgramming::{
        // https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation
        NtQuerySystemInformation as nt_query_system_information,
        SYSTEM_INFORMATION_CLASS,
    },
};

/// The type ID of "File" kernel objects.
const FILE_HANDLE_OBJECT_TYPE_ID: u8 = 37;

// The system handle information request and response are not
// officially documented so we need to write our own type wrappers.

/// A system information class value that retrieves all handles
/// from the kernel.
const SYSTEM_HANDLE_INFORMATION: SYSTEM_INFORMATION_CLASS = 0x10;
const SYSTEM_HANDLE_INFO_BUFFER_SIZE: usize = 262144; // 2^18

/// Information about a kernel object handle.
/// See <https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/ex/sysinfo/handle_table_entry.htm>
#[repr(C)]
#[derive(Debug)]
struct SystemHandleTableEntryInfo {
    /// The ID of the process which holds the handle.
    process_id: u16,
    _creator_back_trace_index: u16,
    /// The type of object described by the handle.
    /// See `FILE_HANDLE_OBJECT_TYPE_ID`.
    object_type_id: u8,
    _handle_attributes: u8,
    _handle: u16,
    _object: *mut c_void,
    _granted_access: u32,
}

/// A vector of all kernel object handles in the system.
/// See <https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/ex/sysinfo/handle.htm>
#[repr(C)]
#[derive(Debug)]
struct SystemHandleInformation {
    number_of_handles: u32,
    /// 1-length arrays are interpreted as any-length arrays.
    /// This value should be used as a pointer and re-cast into a slice
    /// `[SystemHandleTableEntryInfo; number_of_handles]`.
    handles: [SystemHandleTableEntryInfo; 1],
}

impl FdList {
    pub fn list(pid: Pid) -> Result<ProcStats, FshcError> {
        let mut stats = ProcStats::new(pid);

        // Get the list of all open kernel object handles.
        let mut buffer: Vec<usize> = Vec::with_capacity(SYSTEM_HANDLE_INFO_BUFFER_SIZE);
        loop {
            buffer.resize(buffer.len() + SYSTEM_HANDLE_INFO_BUFFER_SIZE, 0);
            let mut return_length: u32 = 0;
            match unsafe {
                nt_query_system_information(
                    SYSTEM_HANDLE_INFORMATION,
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut return_length,
                )
            } {
                // We can't query the size of the list so we query
                // repeatedly, increasing the size of the input buffer
                // linearly on each iteration.
                STATUS_INFO_LENGTH_MISMATCH => continue,
                STATUS_SUCCESS => break,
                other => return Err(format!("Quering the kernel failed with code {other}").into()),
            }
        }
        let handles = unsafe {
            let info = &*(buffer.as_ptr() as *const SystemHandleInformation);
            std::slice::from_raw_parts(info.handles.as_ptr(), info.number_of_handles as usize)
        };

        // Count file object handles belonging to the given process.
        for handle in handles {
            if handle.process_id == pid as u16
                && handle.object_type_id == FILE_HANDLE_OBJECT_TYPE_ID
            {
                stats.file_descriptors += 1;
            }
        }

        Ok(stats)
    }
}
