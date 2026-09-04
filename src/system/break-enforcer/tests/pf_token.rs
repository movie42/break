#![cfg(target_os = "macos")]

use break_enforcer::pf::read_token;

const PFCTL_ENABLE_OUTPUT: &str = "No ALTQ support in kernel
ALTQ related functions disabled
pf enabled
Token : 12345678901234567890
";

#[test]
fn the_token_is_read_from_the_line_pfctl_prints() {
    assert_eq!(
        read_token(PFCTL_ENABLE_OUTPUT).as_deref(),
        Some("12345678901234567890")
    );
}

#[test]
fn the_spacing_around_the_colon_does_not_matter() {
    assert_eq!(read_token("Token: 42\n").as_deref(), Some("42"));
    assert_eq!(read_token("  Token   :   42  \n").as_deref(), Some("42"));
}

#[test]
fn output_without_a_token_line_yields_nothing() {
    assert!(read_token("pf enabled\n").is_none());
    assert!(read_token("").is_none());
}

#[test]
fn a_token_that_is_not_a_number_is_rejected() {
    assert!(read_token("Token : \n").is_none());
    assert!(read_token("Token : none\n").is_none());
}

#[test]
fn a_line_that_merely_mentions_a_token_is_not_read_as_one() {
    assert!(read_token("pfctl: Token file missing : 12345\n").is_none());
}
