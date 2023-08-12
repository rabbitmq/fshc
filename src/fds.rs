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
    pub fn list(pid: Pid) -> Result<ProcStats, FshcError> {
        let info = pidinfo::<BSDInfo>(pid as i32, 0)?;
        let fds = listpidinfo::<ListFDs>(pid as i32, info.pbi_nfiles as usize)?;

        let mut stats = ProcStats::new(pid);
        for fd in fds {
            // libproc returns file descriptor types as numbers,
            // try to convert them
            if let ProcFDType::Socket = fd.proc_fdtype.into() {
                stats.socket_descriptors += 1;
            }
            if let ProcFDType::VNode = fd.proc_fdtype.into() {
                stats.file_descriptors += 1;
            }
        }

        Ok(stats)
    }
}

#[cfg(target_os = "linux")]
impl FdList {
    pub fn list(pid: Pid) -> Result<ProcStats, FshcError> {
        let proc = Process::new(pid as i32)?;
        let all_fds = proc.fd()?;

        let mut stats = ProcStats::new(pid);
        let fds = all_fds
            .flatten()
            .filter(|fd_info| matches!(fd_info.target, FDTarget::Path(_) | FDTarget::Socket(_)));
        for fd in fds {
            match fd.target {
                FDTarget::Path(_) => stats.file_descriptors += 1,
                FDTarget::Socket(_) => stats.socket_descriptors += 1,
                _ => (),
            }
        }

        Ok(stats)
    }
}
