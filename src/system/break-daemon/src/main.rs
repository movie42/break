mod reconcile;

use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use break_enforcer::platform_enforcer;

use crate::reconcile::Outcome;

const DEFAULT_INTERVAL_SECONDS: u64 = 1;

#[derive(Debug, PartialEq, Eq)]
struct Args {
    rules: PathBuf,
    interval: Duration,
    once: bool,
    clear: bool,
}

fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Args, String> {
    let mut rules: Option<PathBuf> = None;
    let mut interval = DEFAULT_INTERVAL_SECONDS;
    let mut once = false;
    let mut clear = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rules" => {
                let value = args.next().ok_or("--rules 뒤에 경로가 없습니다")?;
                rules = Some(PathBuf::from(value));
            }
            "--interval" => {
                let value = args.next().ok_or("--interval 뒤에 초가 없습니다")?;
                interval = value
                    .parse::<u64>()
                    .map_err(|_| format!("--interval 값을 초로 읽을 수 없습니다: {value}"))?;
                if interval == 0 {
                    return Err("--interval 은 1 이상이어야 합니다".to_string());
                }
            }
            "--once" => once = true,
            "--clear" => clear = true,
            other => return Err(format!("모르는 인자입니다: {other}")),
        }
    }

    Ok(Args {
        rules: rules.ok_or("--rules <경로> 가 필요합니다")?,
        interval: Duration::from_secs(interval),
        once,
        clear,
    })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("break-daemon: {message}");
            eprintln!("사용법: break-daemon --rules <경로> [--interval <초>] [--once] [--clear]");
            return ExitCode::FAILURE;
        }
    };

    let enforcer = platform_enforcer();
    println!("break-daemon {} 시작", env!("CARGO_PKG_VERSION"));

    if args.clear {
        let outcome = reconcile::clear(&enforcer);
        println!("{}", outcome.log_line());
        return exit_code(&outcome);
    }

    let mut last: Option<Outcome> = None;
    loop {
        let outcome = reconcile::tick(&enforcer, &args.rules);
        if last.as_ref() != Some(&outcome) {
            println!("{}", outcome.log_line());
            last = Some(outcome.clone());
        }

        if args.once {
            return exit_code(&outcome);
        }

        thread::sleep(args.interval);
    }
}

fn exit_code(outcome: &Outcome) -> ExitCode {
    match outcome {
        Outcome::Failed(_) => ExitCode::FAILURE,
        _ => ExitCode::SUCCESS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, String> {
        parse_args(args.iter().map(|arg| (*arg).to_string()))
    }

    #[test]
    fn rules_path_is_required() {
        assert!(parse(&["--once"]).is_err());
    }

    #[test]
    fn interval_defaults_to_one_second() {
        let args = parse(&["--rules", "/tmp/rules.json"]).expect("parse");
        assert_eq!(args.interval, Duration::from_secs(1));
        assert!(!args.once);
        assert!(!args.clear);
    }

    #[test]
    fn flags_and_values_are_read_in_any_order() {
        let args = parse(&["--once", "--interval", "5", "--rules", "/tmp/r.json"]).expect("parse");
        assert_eq!(args.rules, PathBuf::from("/tmp/r.json"));
        assert_eq!(args.interval, Duration::from_secs(5));
        assert!(args.once);
    }

    #[test]
    fn a_zero_interval_is_rejected() {
        assert!(parse(&["--rules", "/tmp/r.json", "--interval", "0"]).is_err());
    }

    #[test]
    fn an_unknown_argument_is_rejected() {
        assert!(parse(&["--rules", "/tmp/r.json", "--verbose"]).is_err());
    }
}
