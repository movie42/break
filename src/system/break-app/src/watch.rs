use std::thread;
use std::time::Duration;

use break_core::{mail, notice, schedule};
use chrono::Utc;

use crate::commands::{app_mail_account, current_rules};
use crate::daemon::status::{self, DaemonStatus};

const CHECK_EVERY: Duration = Duration::from_secs(5);

fn notify(build: impl FnOnce(&break_core::rules::Rules) -> mail::Mail) {
    let (Ok(rules), Some(account)) = (current_rules(), app_mail_account()) else {
        return;
    };
    let message = build(&rules);
    if let Err(err) = mail::send(&account, &message) {
        eprintln!("알림 메일을 보내지 못했습니다: {err}");
    }
}

pub fn watch_daemon() {
    thread::spawn(|| {
        let mut was_running = status::current() == DaemonStatus::Running;
        loop {
            thread::sleep(CHECK_EVERY);
            let running = status::current() == DaemonStatus::Running;
            if was_running && !running {
                notify(|rules| notice::daemon_stopped(rules, Utc::now()));
            }
            was_running = running;
        }
    });
}

pub fn notify_quit_if_blocking() {
    let Ok(rules) = current_rules() else {
        return;
    };
    if schedule::is_blocking_now(&rules) {
        notify(|rules| notice::app_quit(rules, Utc::now()));
    }
}
