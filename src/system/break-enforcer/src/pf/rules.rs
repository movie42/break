use std::fmt::Write;
use std::net::IpAddr;

pub const ANCHOR_NAME: &str = "com.movie42.break";
pub const ANCHOR_PATH: &str = "/etc/pf.anchors/com.movie42.break";
pub const HEADER: &str = "# 이 파일은 Break가 관리합니다. 직접 고치지 마세요.";

const DOH_PORT: u16 = 443;
const PROTOCOLS: [&str; 2] = ["tcp", "udp"];

pub fn render_anchor(nameservers: &[IpAddr]) -> String {
    if nameservers.is_empty() {
        return String::new();
    }

    let mut out = String::from(HEADER);
    out.push('\n');
    for address in nameservers {
        for protocol in PROTOCOLS {
            let _ = writeln!(
                out,
                "block drop out quick proto {protocol} from any to {address} port {DOH_PORT}"
            );
        }
    }
    out
}
