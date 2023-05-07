use crate::outcome::*;
use procfs::process::{FDTarget, Process};

pub struct FdList;

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
