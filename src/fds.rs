#[cfg(target_os = "windows")]
mod windows;

use crate::outcome::*;
#[cfg(target_os = "macos")]
use libproc::libproc::{
    bsd_info::BSDInfo,
    file_info::{ListFDs, ProcFDType},
    proc_pid::{listpidinfo, pidinfo},
};
#[cfg(target_os = "linux")]
use procfs::process::{FDTarget, Process};

pub struct FdList;

#[cfg(target_os = "macos")]
impl FdList {
    pub fn list_by_type(pid: Pid) -> Result<ProcStats, FshcError> {
        let info = pidinfo::<BSDInfo>(pid as i32, 0)?;
        let fds = listpidinfo::<ListFDs>(pid as i32, info.pbi_nfiles as usize)?;

        let mut stats = ProcStats::new(pid);
        stats.total_descriptors = fds.len() as u32;

        let mut fd_n = 0;
        let mut sd_n = 0;

        for fd in fds {
            // libproc returns file descriptor types as numbers,
            // try to convert them
            if let ProcFDType::Socket = fd.proc_fdtype.into() {
                sd_n += 1;
            }
            if let ProcFDType::VNode = fd.proc_fdtype.into() {
                fd_n += 1;
            }
        }

        stats.socket_descriptors = Some(sd_n);
        stats.file_descriptors = Some(fd_n);

        Ok(stats)
    }

    pub fn list_total(pid: Pid) -> Result<ProcStats, FshcError> {
        let info = pidinfo::<BSDInfo>(pid as i32, 0)?;
        let fds = listpidinfo::<ListFDs>(pid as i32, info.pbi_nfiles as usize)?;

        let stats = ProcStats {
            pid,
            total_descriptors: fds.len() as u32,
            socket_descriptors: None,
            file_descriptors: None,
        };

        Ok(stats)
    }
}

#[cfg(target_os = "linux")]
impl FdList {
    pub fn list_by_type(pid: Pid) -> Result<ProcStats, FshcError> {
        let proc = Process::new(pid as i32)?;
        let all_fds = proc.fd()?.flatten();

        let mut stats = ProcStats::new(pid);

        let mut fd_n = 0;
        let mut sd_n = 0;

        for fd in all_fds {
            stats.total_descriptors += 1;
            match fd.target {
                FDTarget::Path(_) => fd_n += 1,
                FDTarget::Socket(_) => sd_n += 1,
                _ => (),
            }
        }

        stats.file_descriptors = Some(fd_n);
        stats.socket_descriptors = Some(sd_n);

        Ok(stats)
    }

    pub fn list_total(pid: Pid) -> Result<ProcStats, FshcError> {
        let proc = Process::new(pid as i32)?;
        let total_descriptors = proc.fd()?.flatten().count() as u32;

        let stats = ProcStats {
            pid,
            total_descriptors,
            socket_descriptors: None,
            file_descriptors: None,
        };

        Ok(stats)
    }
}
