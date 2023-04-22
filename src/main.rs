use std::io;

use clap::Parser;
use procfs::{process::{Process, FDTarget}, ProcError};
use serde::Serialize;
use serde_json::json;

const PID_LIMIT: u32 = 99_999;

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long)]
    pid: u32
}

#[derive(Debug, Serialize)]
struct ProcStats {
    pid: i32,
    socket_descriptors: u32,
    file_descriptors: u32
}

#[derive(Serialize)]
struct Failure<'a> {
    message: &'a str,
    details: &'a str
}

type FshcResult = Result<ProcStats, Box<dyn std::error::Error>>;

fn main() {
    let args = CliArgs::parse();
    let res = run(&args);
    
    match res {
        Ok(stats) => {
            println!("{}", json!(stats));
        },
        Err(err) => {
            let failure = Failure {
                message: &format!("Failed to obtain file and socket descriptors of process {}", args.pid),
                details: &err.to_string()
            };
            eprintln!("{}", json!(failure));
            std::process::exit(sysexits::ExitCode::OsErr as i32);
        }
    }
}

fn run(args: &CliArgs) -> FshcResult {
    let pid = validate_pid(args)?;
    let proc = Process::new(pid)?;
    let all_fds = proc.fd()?;
    
    let mut stats = ProcStats {
        pid: pid,
        socket_descriptors: 0,
        file_descriptors: 0
    };
    let fds = all_fds.flatten().filter(|fd_info| {
        match fd_info.target {
            FDTarget::Path(_) => true,
            FDTarget::Socket(_) => true,
            _ => false
        }
    });
    for fd in fds {
        match fd.target {
            FDTarget::Path(_) => stats.file_descriptors += 1,
            FDTarget::Socket(_) => stats.socket_descriptors += 1,
            _ => ()
        }
    }
    
    Ok(stats)
}

fn validate_pid(args: &CliArgs) -> Result<i32, Box<dyn std::error::Error>> {
    if args.pid > PID_LIMIT {
        eprintln!("pids greater than 99999 are not supported, provided: {}", args.pid);
        let err = Box::new(io::Error::new(io::ErrorKind::InvalidInput, "pids greater than 99999 are not supported"));
        return Err(err);
    }

    match TryInto::<i32>::try_into(args.pid) {
        Ok(val) => Ok(val),
        Err(_) => {
            let err = Box::new(io::Error::new(io::ErrorKind::InvalidInput, "pids greater than 99999 are not supported"));
            Err(err)
        }
    }
}