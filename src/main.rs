mod fds;
mod outcome;

use clap::Parser;
use serde::Serialize;
use std::fmt;
use sysexits::ExitCode;

use crate::fds::*;
use crate::outcome::*;

const PID_LIMIT: u32 = 99_999;

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long)]
    pid: u32,
}

fn main() {
    let args = CliArgs::parse();
    let res = run(&args);

    terminate(res, &args)
}

fn run(args: &CliArgs) -> FshcResult {
    let pid = validate_pid(args)?;
    let stats = FdList::list(pid)?;

    Ok(stats)
}

fn terminate(outcome: FshcResult, args: &CliArgs) {
    match outcome {
        Ok(stats) => {
            exit(stats, ExitCode::Ok);
        }
        Err(err) => {
            let failure = Failure {
                message: &format!(
                    "Failed to obtain file and socket descriptors of process {}",
                    args.pid
                ),
                details: &err.to_string(),
            };
            exit(failure, err.exit_code());
        }
    }
}

fn exit<T: Serialize + fmt::Debug>(data: T, code: ExitCode) {
    match code {
        ExitCode::Ok => {
            println!(
                "{}",
                serde_json::to_string(&data)
                    .unwrap_or_else(|err| panic!("could not serialize {:?}: {}", data, err))
            );
            std::process::exit(code as i32);
        }
        _ => {
            eprintln!(
                "{}",
                serde_json::to_string(&data)
                    .unwrap_or_else(|err| panic!("could not serialize {:?}: {}", data, err))
            );
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
        Err(_) => Err(FshcError::InvalidInput),
    }
}
