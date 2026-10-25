mod peer;
mod reconcile;
mod server;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use break_core::events::{self, EventKind, HEARTBEAT_SECONDS};
use break_core::protocol::SOCKET_PATH;
use break_core::rules::SiteTarget;
use break_core::{change, mail, notice, store};
use break_enforcer::platform_enforcer;
use chrono::Utc;

use crate::reconcile::Outcome;
use crate::server::Config;

const DEFAULT_INTERVAL_SECONDS: u64 = 1;
pub const ADULT_LIST_PATH: &str = "/Library/Application Support/Break/adult-hosts.txt";

#[derive(Debug, PartialEq, Eq)]
struct Args {
    rules: PathBuf,
    import: Option<PathBuf>,
    socket: PathBuf,
    events: PathBuf,
    heartbeat: PathBuf,
    mail_account: PathBuf,
    adult_list: PathBuf,
    clients: Vec<PathBuf>,
    release_delay_seconds: i64,
    interval: Duration,
    once: bool,
    clear: bool,
}

fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Args, String> {
    let mut rules: Option<PathBuf> = None;
    let mut import = None;
    let mut socket = PathBuf::from(SOCKET_PATH);
    let mut events_path = PathBuf::from(events::EVENTS_PATH);
    let mut heartbeat = PathBuf::from(events::HEARTBEAT_PATH);
    let mut mail_account = PathBuf::from(mail::ACCOUNT_PATH);
    let mut adult_list = PathBuf::from(ADULT_LIST_PATH);
    let mut clients = Vec::new();
    let mut release_delay_seconds = change::RELEASE_DELAY_SECONDS;
    let mut interval = DEFAULT_INTERVAL_SECONDS;
    let mut once = false;
    let mut clear = false;

    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} 뒤에 값이 없습니다"));
        match arg.as_str() {
            "--rules" => rules = Some(PathBuf::from(value("--rules")?)),
            "--import" => import = Some(PathBuf::from(value("--import")?)),
            "--socket" => socket = PathBuf::from(value("--socket")?),
            "--events" => events_path = PathBuf::from(value("--events")?),
            "--heartbeat" => heartbeat = PathBuf::from(value("--heartbeat")?),
            "--mail-account" => mail_account = PathBuf::from(value("--mail-account")?),
            "--adult-list" => adult_list = PathBuf::from(value("--adult-list")?),
            "--client" => clients.push(PathBuf::from(value("--client")?)),
            "--release-delay" => {
                let raw = value("--release-delay")?;
                release_delay_seconds = raw
                    .parse::<i64>()
                    .ok()
                    .filter(|seconds| *seconds > 0)
                    .ok_or(format!("--release-delay 값을 초로 읽을 수 없습니다: {raw}"))?;
            }
            "--interval" => {
                let raw = value("--interval")?;
                interval = raw
                    .parse::<u64>()
                    .map_err(|_| format!("--interval 값을 초로 읽을 수 없습니다: {raw}"))?;
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
        import,
        socket,
        events: events_path,
        heartbeat,
        mail_account,
        adult_list,
        clients,
        release_delay_seconds,
        interval: Duration::from_secs(interval),
        once,
        clear,
    })
}

fn list_host(raw: &str) -> Option<String> {
    let host = raw.trim_end_matches('.').to_ascii_lowercase();
    let valid = host.contains('.')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
    valid.then_some(host)
}

fn load_adult_list(path: &Path) -> Vec<SiteTarget> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut sites: Vec<SiteTarget> = raw
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default().trim())
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_whitespace().last())
        .filter_map(list_host)
        .map(|host| SiteTarget { host })
        .collect();
    sites.sort_by(|left, right| left.host.cmp(&right.host));
    sites.dedup();
    sites
}

fn import_user_rules(rules_path: &Path, import: Option<&Path>) {
    let Some(import) = import else {
        return;
    };
    if rules_path.exists() || !import.exists() {
        return;
    }
    match store::load_from(import) {
        Ok(rules) => match store::save_atomic(rules_path, &rules, store::SYSTEM_RULES_MODE) {
            Ok(()) => {
                let _ = fs::remove_file(import);
                println!("사용자 폴더의 규칙을 옮겼습니다: {}", import.display());
            }
            Err(err) => eprintln!("규칙을 옮기지 못했습니다: {err}"),
        },
        Err(err) => eprintln!("옮길 규칙을 읽지 못했습니다: {err}"),
    }
}

fn report_downtime(config: &Arc<Config>, heartbeat: &Path) {
    let now = Utc::now();
    let Some((from, to)) = events::downtime(events::read_heartbeat(heartbeat), now) else {
        return;
    };
    server::log_event(config, EventKind::Down { from, to });
    if let Ok(rules) = store::load_from(&config.rules_path) {
        server::send_later(
            Arc::clone(config),
            notice::daemon_was_down(&rules, from, to),
        );
    }
}

fn release_due(config: &Config, rules_lock: &Mutex<()>) {
    let Ok(_guard) = rules_lock.lock() else {
        return;
    };
    let Ok(mut rules) = store::load_from(&config.rules_path) else {
        return;
    };
    let released = change::apply_due_releases(&mut rules, Utc::now(), config.release_delay);
    if released.is_empty() {
        return;
    }
    if store::save_atomic(&config.rules_path, &rules, store::SYSTEM_RULES_MODE).is_ok() {
        for message in released {
            server::log_event(config, EventKind::Released { message });
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("break-daemon: {message}");
            eprintln!(
                "사용법: break-daemon --rules <경로> [--import <경로>] [--interval <초>] [--once] [--clear]"
            );
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

    let adult = load_adult_list(&args.adult_list);

    if args.once {
        let (outcome, _) = reconcile::tick_report(&enforcer, &args.rules, &adult);
        println!("{}", outcome.log_line());
        return exit_code(&outcome);
    }

    import_user_rules(&args.rules, args.import.as_deref());

    let config = Arc::new(Config {
        rules_path: args.rules.clone(),
        socket_path: args.socket.clone(),
        events_path: args.events.clone(),
        mail_account_path: args.mail_account.clone(),
        clients: args.clients.clone(),
        release_delay: chrono::Duration::seconds(args.release_delay_seconds),
    });
    let rules_lock = Arc::new(Mutex::new(()));

    report_downtime(&config, &args.heartbeat);
    server::log_event(&config, EventKind::Started);
    if let Err(err) = server::start(Arc::clone(&config), Arc::clone(&rules_lock)) {
        eprintln!("요청 소켓을 열지 못했습니다: {err}");
    }

    let beat_every = u64::try_from(HEARTBEAT_SECONDS).unwrap_or(5);
    let mut last: Option<Outcome> = None;
    let mut seconds: u64 = 0;
    loop {
        release_due(&config, &rules_lock);

        let (outcome, hosts_changed) = reconcile::tick_report(&enforcer, &args.rules, &adult);
        if last.as_ref() == Some(&outcome) && hosts_changed {
            server::log_event(&config, EventKind::HostsRestored);
        }
        if last.as_ref() != Some(&outcome) {
            println!("{}", outcome.log_line());
            last = Some(outcome);
        }

        if seconds % beat_every == 0 {
            let _ = events::write_heartbeat(&args.heartbeat, Utc::now());
        }
        seconds += args.interval.as_secs();

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
        assert_eq!(args.socket, PathBuf::from(SOCKET_PATH));
        assert_eq!(args.import, None);
        assert_eq!(args.release_delay_seconds, change::RELEASE_DELAY_SECONDS);
    }

    #[test]
    fn a_test_build_can_shorten_the_release_delay() {
        let args = parse(&["--rules", "/tmp/r.json", "--release-delay", "60"]).expect("parse");
        assert_eq!(args.release_delay_seconds, 60);
        assert!(parse(&["--rules", "/tmp/r.json", "--release-delay", "0"]).is_err());
    }

    #[test]
    fn flags_and_values_are_read_in_any_order() {
        let args = parse(&[
            "--once",
            "--interval",
            "5",
            "--import",
            "/tmp/old.json",
            "--rules",
            "/tmp/r.json",
        ])
        .expect("parse");
        assert_eq!(args.rules, PathBuf::from("/tmp/r.json"));
        assert_eq!(args.import, Some(PathBuf::from("/tmp/old.json")));
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

    #[test]
    fn the_adult_list_reads_hosts_lines_and_skips_comments() {
        let dir = std::env::temp_dir().join(format!("break-adult-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("list.txt");
        fs::write(
            &path,
            "# title\n0.0.0.0 b.example\n0.0.0.0 a.example # note\nc.example\n\n0.0.0.0 a.example\n0.0.0.0 www.a.example\n",
        )
        .expect("write");

        let hosts: Vec<String> = load_adult_list(&path)
            .into_iter()
            .map(|site| site.host)
            .collect();
        assert_eq!(
            hosts,
            vec!["a.example", "b.example", "c.example", "www.a.example"]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_adult_list_is_empty() {
        assert!(load_adult_list(Path::new("/nonexistent/break/list.txt")).is_empty());
    }
}
