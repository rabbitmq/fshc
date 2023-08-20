use super::*;

use std::ffi::c_void;
use windows_sys::Win32::{
    Foundation::{HANDLE, STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS, UNICODE_STRING},
    System::{
        // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocessid
        Threading::GetCurrentProcessId as get_current_process_id,
        WindowsProgramming::{
            // https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject
            NtQueryObject as nt_query_object,
            // https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation
            NtQuerySystemInformation as nt_query_system_information,
            OBJECT_INFORMATION_CLASS,
            SYSTEM_INFORMATION_CLASS,
        },
    },
};

// The system handle information request and response are not
// officially documented so we need to write our own type wrappers.

/// A system information class value that retrieves all handles
/// from the kernel.
const SYSTEM_HANDLE_INFORMATION: SYSTEM_INFORMATION_CLASS = 0x10;
const SYSTEM_HANDLE_INFO_BUFFER_SIZE: usize = 262144; // 2^18

/// An object information class that retrieves the name of the
/// kernel object's type.
const OBJECT_TYPE_INFORMATION: OBJECT_INFORMATION_CLASS = 0x2;
const OBJECT_INFO_BUFFER_SIZE: usize = 4096; // 2^12

/// `"File"` encoded as UTF16.
/// `str` is UTF8-encoded but `UNICODE_STRING` is UTF16.
const FILE_HANDLE_NAME: &[u16; 4] = &[70, 105, 108, 101];

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
    handle: u16,
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

/// The name of the kind of a kernel object handle.
/// See <https://www.geoffchappell.com/studies/windows/km/ntoskrnl/inc/api/ntobapi/object_type_information.htm>
#[repr(C)]
struct ObjectTypeInformation {
    type_name: UNICODE_STRING,
    _reserved: [u32; 22],
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
                other => {
                    return Err(format!("Failed to query for all handles with code {other}").into())
                }
            }
        }
        let handles = unsafe {
            let info = &*(buffer.as_ptr() as *const SystemHandleInformation);
            std::slice::from_raw_parts(info.handles.as_ptr(), info.number_of_handles as usize)
        };

        let current_process_id = unsafe { get_current_process_id() } as u16;
        let file_handle_object_type_id = handles
            .iter()
            .filter(|handle| handle.process_id == current_process_id)
            .find_map(|handle| {
                let buffer = [0; OBJECT_INFO_BUFFER_SIZE].as_mut_ptr();
                let mut return_length: u32 = 0;
                match unsafe {
                    nt_query_object(
                        handle.handle as HANDLE,
                        OBJECT_TYPE_INFORMATION,
                        buffer as *mut c_void,
                        OBJECT_INFO_BUFFER_SIZE as u32,
                        &mut return_length,
                    )
                } {
                    STATUS_SUCCESS => {
                        let name_units = unsafe {
                            let info = buffer.cast::<ObjectTypeInformation>();
                            let name = std::ptr::addr_of!((*info).type_name).read_unaligned();
                            std::slice::from_raw_parts(name.Buffer, name.Length as usize)
                        };
                        if &name_units[0..4.min(name_units.len())] == FILE_HANDLE_NAME {
                            Some(Ok(handle.object_type_id))
                        } else {
                            None
                        }
                    }
                    other => Some(Err(FshcError::from(format!(
                        "Failed to query handle information with code {other}"
                    )))),
                }
            })
            .ok_or_else(|| {
                FshcError::from("Failed to find file handles in the current process".to_string())
            })??;

        let pid = pid as u16;
        stats.file_descriptors = handles
            .iter()
            .filter(|handle| {
                handle.process_id == pid && handle.object_type_id == file_handle_object_type_id
            })
            .count() as u32;

        Ok(stats)
    }
}
