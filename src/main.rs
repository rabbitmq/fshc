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
#[command(version = clap::crate_version!(), about = "File and socket handle counter", long_about = None, after_help = "GitHub: https://github.com/rabbitmq/fshc")]
struct CliArgs {
    #[arg(long)]
    only_total: bool,
    #[arg(short, long)]
    pid: u32,
}

fn main() -> ExitCode {
    let args = CliArgs::parse();
    let res = run(&args);

    terminate(res, &args)
}

fn run(args: &CliArgs) -> FshcResult {
    let pid = validate_pid(args)?;
    let stats = if args.only_total {
        FdList::list_total(pid)?
    } else {
        FdList::list_by_type(pid)?
    };

    Ok(stats)
}

fn terminate(outcome: FshcResult, args: &CliArgs) -> ExitCode {
    match outcome {
        Ok(stats) => exit(stats, ExitCode::Ok),
        Err(err) => {
            let failure = Failure {
                message: &format!(
                    "Failed to obtain file and socket descriptors of process {}",
                    args.pid
                ),
                details: &err.to_string(),
            };
            exit(failure, err.exit_code())
        }
    }
}

fn exit<T: Serialize + fmt::Debug>(data: T, code: ExitCode) -> ExitCode {
    let output = serde_json::to_string(&data)
        .unwrap_or_else(|err| panic!("could not serialize {:?}: {}", data, err));
    match code {
        ExitCode::Ok => println!("{}", output),
        _ => eprintln!("{}", output),
    }
    code
}

fn validate_pid(args: &CliArgs) -> Result<Pid, FshcError> {
    if (1..=PID_LIMIT).contains(&args.pid) {
        Ok(args.pid)
    } else {
        Err(FshcError::PidOutOfRange)
    }
}
