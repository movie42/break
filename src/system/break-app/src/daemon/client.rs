use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use break_core::protocol::{Request, Response, SOCKET_PATH};

const TIMEOUT: Duration = Duration::from_secs(10);

pub fn reachable() -> bool {
    Path::new(SOCKET_PATH).exists() && send(&Request::Ping).is_ok()
}

pub fn send(request: &Request) -> Result<Response, String> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .map_err(|err| format!("차단 프로그램에 연결하지 못했습니다: {err}"))?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|err| err.to_string())?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|err| err.to_string())?;

    let json = serde_json::to_string(request).map_err(|err| err.to_string())?;
    writeln!(stream, "{json}").map_err(|err| err.to_string())?;

    let mut line = String::new();
    BufReader::new(&stream)
        .read_line(&mut line)
        .map_err(|err| format!("차단 프로그램의 대답을 읽지 못했습니다: {err}"))?;
    serde_json::from_str(&line)
        .map_err(|err| format!("차단 프로그램의 대답을 읽지 못했습니다: {err}"))
}
