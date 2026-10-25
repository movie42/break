use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use break_core::events::{self, Event, EventKind};
use break_core::protocol::{self, Request, Response};
use break_core::{mail, notice, store};
use chrono::Utc;

const MAX_REQUEST_BYTES: u64 = 4 * 1024 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const SOCKET_MODE: u32 = 0o666;

pub struct Config {
    pub rules_path: PathBuf,
    pub socket_path: PathBuf,
    pub events_path: PathBuf,
    pub mail_account_path: PathBuf,
    pub clients: Vec<PathBuf>,
    pub release_delay: chrono::Duration,
}

pub fn log_event(config: &Config, kind: EventKind) {
    let event = Event {
        at: Utc::now(),
        kind,
    };
    if let Err(err) = events::append(&config.events_path, &event) {
        eprintln!("기록을 남기지 못했습니다: {err}");
    }
}

pub fn send_later(config: Arc<Config>, message: mail::Mail) {
    if message.to.is_empty() {
        return;
    }
    thread::spawn(move || {
        let Some(account) = mail::load_account(&config.mail_account_path) else {
            return;
        };
        if let Err(err) = mail::send(&account, &message) {
            log_event(&config, EventKind::MailFailed { message: err });
        }
    });
}

pub fn start(config: Arc<Config>, rules_lock: Arc<Mutex<()>>) -> std::io::Result<()> {
    let _ = fs::remove_file(&config.socket_path);
    let listener = UnixListener::bind(&config.socket_path)?;
    fs::set_permissions(&config.socket_path, fs::Permissions::from_mode(SOCKET_MODE))?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else {
                continue;
            };
            let config = Arc::clone(&config);
            let rules_lock = Arc::clone(&rules_lock);
            thread::spawn(move || serve_one(stream, &config, &rules_lock));
        }
    });
    Ok(())
}

fn serve_one(stream: UnixStream, config: &Arc<Config>, rules_lock: &Mutex<()>) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let mut line = String::new();
    let read = BufReader::new(&stream)
        .take(MAX_REQUEST_BYTES)
        .read_line(&mut line);

    let peer = crate::peer::executable(&stream);
    let trusted = crate::peer::allowed(peer.as_deref(), &config.clients);

    let response = match read {
        Ok(_) => match serde_json::from_str::<Request>(&line) {
            Ok(Request::Ping) => Response::ok(),
            Ok(_) if !trusted => {
                log_event(
                    config,
                    EventKind::Request {
                        message: format!(
                            "Break 앱이 아닌 프로그램의 요청을 거절했습니다: {}",
                            peer.as_deref()
                                .map(|path| path.display().to_string())
                                .unwrap_or_else(|| "알 수 없음".to_string())
                        ),
                    },
                );
                Response::refused("Break 앱에서 보낸 요청만 받습니다")
            }
            Ok(request) => handle(request, config, rules_lock),
            Err(err) => Response::refused(format!("요청을 읽지 못했습니다: {err}")),
        },
        Err(err) => Response::refused(format!("요청을 읽지 못했습니다: {err}")),
    };

    let mut writer = &stream;
    if let Ok(json) = serde_json::to_string(&response) {
        let _ = writeln!(writer, "{json}");
    }
}

fn handle(request: Request, config: &Arc<Config>, rules_lock: &Mutex<()>) -> Response {
    let Ok(_guard) = rules_lock.lock() else {
        return Response::refused("차단 프로그램이 바쁩니다. 다시 시도하세요");
    };

    let rules = match store::load_from(&config.rules_path) {
        Ok(rules) => rules,
        Err(err) => return Response::refused(err.to_string()),
    };

    match request {
        Request::Ping => Response::ok(),
        Request::SetMailAccount { account, password } => {
            let replacing = mail::load_account(&config.mail_account_path).is_some();
            if replacing {
                let verified = password
                    .as_deref()
                    .is_some_and(|input| protocol::verify_password(&rules, input));
                if !verified {
                    return Response::refused("메일 계정을 바꾸려면 비밀번호가 필요합니다");
                }
            }
            match mail::save_account(&config.mail_account_path, &account) {
                Ok(()) => {
                    log_event(
                        config,
                        EventKind::Request {
                            message: format!("메일 계정을 {}로 정했습니다", account.from),
                        },
                    );
                    Response::ok()
                }
                Err(err) => Response::refused(format!("메일 계정을 저장하지 못했습니다: {err}")),
            }
        }
        request => {
            let mut next = rules.clone();
            let now = Utc::now();
            let applied = match protocol::apply(&mut next, request, now) {
                Ok(applied) => applied,
                Err(err) => return Response::refused(err.to_string()),
            };
            if let Err(err) =
                store::save_atomic(&config.rules_path, &next, store::SYSTEM_RULES_MODE)
            {
                return Response::refused(err.to_string());
            }

            log_event(
                config,
                EventKind::Request {
                    message: applied.event,
                },
            );
            if let Some(email) = applied.new_recipient {
                send_later(Arc::clone(config), notice::confirmation(&email));
            }
            if let Some(email) = applied.removal_notice {
                send_later(
                    Arc::clone(config),
                    notice::removal_planned(&next, &email, now + config.release_delay),
                );
            }
            Response::ok()
        }
    }
}
