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
use procfs::process::{FDInfo, FDTarget, Process};

pub struct FdList;

#[cfg(target_os = "macos")]
impl FdList {
    pub fn list(pid: Pid, only_total: bool) -> Result<ProcStats, FshcError> {
        let info = pidinfo::<BSDInfo>(pid as i32, 0)?;
        let fds = listpidinfo::<ListFDs>(pid as i32, info.pbi_nfiles as usize)?;

        let mut stats = ProcStats::new(pid);
        stats.total_descriptors = fds.len() as u32;

        if !only_total {
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
        }

        Ok(stats)
    }
}

#[cfg(target_os = "linux")]
impl FdList {
    pub fn list(pid: Pid, only_total: bool) -> Result<ProcStats, FshcError> {
        let proc = Process::new(pid as i32)?;
        let all_fds = proc.fd()?.flatten().collect::<Vec<FDInfo>>();

        let mut stats = ProcStats::new(pid);
        stats.total_descriptors = all_fds.len() as u32;

        if !only_total {
            let mut fd_n = 0;
            let mut sd_n = 0;

            let fds = all_fds.iter().filter(|fd_info| {
                matches!(fd_info.target, FDTarget::Path(_) | FDTarget::Socket(_))
            });
            for fd in fds {
                match fd.target {
                    FDTarget::Path(_) => fd_n += 1,
                    FDTarget::Socket(_) => sd_n += 1,
                    _ => (),
                }
            }

            stats.file_descriptors = Some(fd_n);
            stats.socket_descriptors = Some(sd_n);
        }

        Ok(stats)
    }
}
