use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub const EVENTS_PATH: &str = "/Library/Logs/Break/events.jsonl";
pub const HEARTBEAT_PATH: &str = "/Library/Application Support/Break/heartbeat";
pub const HEARTBEAT_SECONDS: i64 = 5;
pub const DOWN_AFTER_SECONDS: i64 = 15;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum EventKind {
    Started,
    Stopped,
    Down {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    },
    HostsRestored,
    RulesChanged,
    Request {
        message: String,
    },
    Released {
        message: String,
    },
    MailFailed {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub at: DateTime<Utc>,
    #[serde(flatten)]
    pub kind: EventKind,
}

pub fn append(path: &Path, event: &Event) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(event).map_err(io::Error::other)?;
    writeln!(file, "{line}")
}

pub fn read_recent(path: &Path, limit: usize) -> Vec<Event> {
    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };
    let mut events: Vec<Event> = BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect();
    let skip = events.len().saturating_sub(limit);
    events.drain(..skip);
    events.reverse();
    events
}

pub fn read_heartbeat(path: &Path) -> Option<DateTime<Utc>> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn write_heartbeat(path: &Path, at: DateTime<Utc>) -> io::Result<()> {
    fs::write(path, at.to_rfc3339())
}

pub fn downtime(
    last_beat: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let last = last_beat?;
    (now - last > Duration::seconds(DOWN_AFTER_SECONDS)).then_some((last, now))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        text.parse().expect("time")
    }

    #[test]
    fn a_short_gap_is_not_downtime() {
        assert_eq!(
            downtime(Some(at("2026-10-18T10:00:00Z")), at("2026-10-18T10:00:10Z")),
            None
        );
    }

    #[test]
    fn a_long_gap_is_reported_from_the_last_beat() {
        let from = at("2026-10-18T10:04:00Z");
        let to = at("2026-10-18T10:31:00Z");
        assert_eq!(downtime(Some(from), to), Some((from, to)));
    }

    #[test]
    fn no_heartbeat_means_a_first_start() {
        assert_eq!(downtime(None, at("2026-10-18T10:00:00Z")), None);
    }

    #[test]
    fn events_come_back_newest_first_and_bad_lines_are_skipped() {
        let dir = std::env::temp_dir().join(format!("break-events-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("events.jsonl");
        append(
            &path,
            &Event {
                at: at("2026-10-18T10:00:00Z"),
                kind: EventKind::Started,
            },
        )
        .expect("append");
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .and_then(|mut file| writeln!(file, "not json"))
            .expect("garbage");
        append(
            &path,
            &Event {
                at: at("2026-10-18T10:01:00Z"),
                kind: EventKind::RulesChanged,
            },
        )
        .expect("append");

        let events = read_recent(&path, 10);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, EventKind::RulesChanged);
        let _ = fs::remove_dir_all(&dir);
    }
}
