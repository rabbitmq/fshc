mod outcome;

use std::fmt;
use clap::Parser;
use procfs::{process::{Process, FDTarget}};
use serde::Serialize;
use sysexits::ExitCode;

use crate::outcome::*;

const PID_LIMIT: u32 = 99_999;

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long)]
    pid: u32
}

fn main() {
    let args = CliArgs::parse();
    let res = run(&args);
    
    terminate(res, &args)
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

fn terminate(outcome: FshcResult, args: &CliArgs) {
    match outcome {
        Ok(stats) => {
            exit(&stats, ExitCode::Ok);
        },
        Err(err) => {
            let failure = Failure {
                message: &format!("Failed to obtain file and socket descriptors of process {}", args.pid),
                details: &err.to_string()
            };
            exit(&failure, err.exit_code());
        }
    }
}

fn exit<T: Serialize + fmt::Debug>(data: T, code: ExitCode) {
    match code {
        ExitCode::Ok => {
            println!("{}", serde_json::to_string(&data).expect(&format!("could not serialize {:?}", data)));
            std::process::exit(code as i32);
        },
        _ => {
            eprintln!("{}", serde_json::to_string(&data).expect(&format!("could not serialize {:?}", data)));
            std::process::exit(code as i32);
        }
    }
}

fn validate_pid(args: &CliArgs) -> Result<i32, FshcError> {
    if args.pid > PID_LIMIT {
        return Err(FshcError::PidOutOfRange);
    }

    match TryInto::<i32>::try_into(args.pid) {
        Ok(val) => Ok(val),
        Err(_) => {
            Err(FshcError::InvalidInput)
        }
    }
}