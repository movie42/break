use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

const SOL_LOCAL: libc::c_int = 0;
const PATH_BUFFER: usize = 4096;

pub fn executable(stream: &UnixStream) -> Option<PathBuf> {
    let mut pid: libc::pid_t = 0;
    let mut length = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
    let found = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            SOL_LOCAL,
            libc::LOCAL_PEERPID,
            std::ptr::addr_of_mut!(pid).cast(),
            &mut length,
        )
    };
    if found != 0 || pid <= 0 {
        return None;
    }

    let mut buffer = vec![0u8; PATH_BUFFER];
    let written =
        unsafe { libc::proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if written <= 0 {
        return None;
    }
    buffer.truncate(usize::try_from(written).ok()?);
    String::from_utf8(buffer).ok().map(PathBuf::from)
}

pub fn allowed(peer: Option<&Path>, clients: &[PathBuf]) -> bool {
    if clients.is_empty() {
        return true;
    }
    peer.is_some_and(|path| clients.iter().any(|client| client == path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_registered_client_lets_any_peer_through() {
        assert!(allowed(None, &[]));
    }

    #[test]
    fn only_the_registered_app_is_allowed() {
        let app = PathBuf::from("/Applications/Break.app/Contents/MacOS/break-app");
        let clients = vec![app.clone()];
        assert!(allowed(Some(&app), &clients));
        assert!(!allowed(Some(Path::new("/usr/bin/nc")), &clients));
        assert!(!allowed(None, &clients));
    }

    #[test]
    fn a_socket_pair_reports_this_test_binary() {
        let (left, _right) = UnixStream::pair().expect("pair");
        let peer = executable(&left).expect("peer path");
        assert_eq!(peer, std::env::current_exe().expect("exe"));
    }
}
