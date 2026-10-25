use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::protocol::MailAccount;

pub const ACCOUNT_PATH: &str = "/Library/Application Support/Break/mail.json";
const SMTPS_PORT: u16 = 465;
const CURL_TIMEOUT_SECONDS: &str = "20";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(input: &[u8]) -> String {
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let bytes = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let packed = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
        for index in 0..4 {
            if index <= chunk.len() {
                let sextet = (packed >> (18 - 6 * index)) & 0x3f;
                out.push(char::from(BASE64[sextet as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn encoded_word(text: &str) -> String {
    format!("=?UTF-8?B?{}?=", base64(text.as_bytes()))
}

pub fn render(account: &MailAccount, mail: &Mail) -> String {
    let body = base64(mail.body.as_bytes());
    let wrapped: Vec<&str> = body
        .as_bytes()
        .chunks(76)
        .map(|line| std::str::from_utf8(line).unwrap_or_default())
        .collect();
    format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n{body}\r\n",
        from = account.from,
        to = mail.to.join(", "),
        subject = encoded_word(&mail.subject),
        body = wrapped.join("\r\n"),
    )
}

fn url(account: &MailAccount) -> String {
    let scheme = if account.port == SMTPS_PORT {
        "smtps"
    } else {
        "smtp"
    };
    format!("{scheme}://{}:{}", account.host, account.port)
}

fn quote_config(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn credentials_file(account: &MailAccount) -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!(
        "break-mail-{}-{}.conf",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    writeln!(
        file,
        "user = {}",
        quote_config(&format!("{}:{}", account.username, account.secret))
    )?;
    Ok(path)
}

pub fn load_account(path: &std::path::Path) -> Option<MailAccount> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

pub fn save_account(path: &std::path::Path, account: &MailAccount) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let json = serde_json::to_string(account).map_err(std::io::Error::other)?;
    let temp = path.with_extension("tmp");
    fs::write(&temp, json)?;
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))?;
    fs::rename(&temp, path)
}

pub fn send(account: &MailAccount, mail: &Mail) -> Result<(), String> {
    if mail.to.is_empty() {
        return Ok(());
    }

    let config = credentials_file(account).map_err(|err| err.to_string())?;
    let mut command = Command::new("curl");
    command
        .args(["--silent", "--show-error", "--ssl-reqd"])
        .args(["--max-time", CURL_TIMEOUT_SECONDS])
        .args(["--url", &url(account)])
        .args(["--mail-from", &account.from])
        .arg("--config")
        .arg(&config)
        .args(["--upload-file", "-"]);
    for recipient in &mail.to {
        command.args(["--mail-rcpt", recipient]);
    }

    let result = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(render(account, mail).as_bytes())?;
            }
            child.wait_with_output()
        });
    let _ = fs::remove_file(&config);

    match result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account() -> MailAccount {
        MailAccount {
            host: "smtp.gmail.com".to_string(),
            port: 465,
            username: "me@example.com".to_string(),
            secret: "app-secret".to_string(),
            from: "me@example.com".to_string(),
        }
    }

    #[test]
    fn base64_matches_the_standard_padding() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64("차단".as_bytes()), "7LCo64uo");
    }

    #[test]
    fn a_korean_subject_is_encoded_and_the_secret_never_appears() {
        let text = render(
            &account(),
            &Mail {
                to: vec!["friend@example.com".to_string()],
                subject: "차단 프로그램이 꺼졌습니다".to_string(),
                body: "19:04부터".to_string(),
            },
        );
        assert!(text.contains("Subject: =?UTF-8?B?"));
        assert!(text.contains("To: friend@example.com"));
        assert!(!text.contains("app-secret"));
    }

    #[test]
    fn port_465_uses_implicit_tls() {
        assert_eq!(url(&account()), "smtps://smtp.gmail.com:465");
        let mut starttls = account();
        starttls.port = 587;
        assert_eq!(url(&starttls), "smtp://smtp.gmail.com:587");
    }

    #[test]
    fn a_quote_in_the_secret_stays_inside_the_config_value() {
        assert_eq!(quote_config(r#"a"b\c"#), r#""a\"b\\c""#);
    }
}
