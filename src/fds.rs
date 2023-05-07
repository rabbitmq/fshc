use crate::outcome::*;
use libproc::libproc::file_info::ProcFDType;
#[cfg(target_os = "macos")]
use libproc::libproc::{
    bsd_info::BSDInfo,
    file_info::ListFDs,
    proc_pid::{listpidinfo, pidinfo},
};
#[cfg(target_os = "linux")]
use procfs::process::{FDTarget, Process};

pub struct FdList;

#[cfg(target_os = "macos")]
impl FdList {
    pub fn list(pid: i32) -> Result<ProcStats, FshcError> {
        let info = pidinfo::<BSDInfo>(pid, 0)?;
        let fds = listpidinfo::<ListFDs>(pid, info.pbi_nfiles as usize)?;

        let mut stats = ProcStats {
            pid: pid,
            socket_descriptors: 0,
            file_descriptors: 0,
        };
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
    pub fn list(pid: i32) -> Result<ProcStats, FshcError> {
        let proc = Process::new(pid)?;
        let all_fds = proc.fd()?;

        let mut stats = ProcStats {
            pid: pid,
            socket_descriptors: 0,
            file_descriptors: 0,
        };
        let fds = all_fds.flatten().filter(|fd_info| match fd_info.target {
            FDTarget::Path(_) => true,
            FDTarget::Socket(_) => true,
            _ => false,
        });
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
