#![cfg(target_os = "macos")]

use std::net::IpAddr;

use break_enforcer::pf::resolvers::{blockable, parse_nameservers};
use break_enforcer::pf::rules::{render_anchor, HEADER};

const SCUTIL_SAMPLE: &str = "DNS configuration

resolver #1
  search domain[0] : local
  nameserver[0] : 192.168.0.1
  nameserver[1] : 1.1.1.1
  flags    : Request A records, Request AAAA records
  reach    : 0x00000002 (Reachable)

resolver #2
  domain   : local
  options  : mdns
  timeout  : 5
  flags    : Request A records
";

fn addresses(values: &[&str]) -> Vec<IpAddr> {
    values.iter().map(|value| value.parse().expect("ip")).collect()
}

#[test]
fn nameservers_are_read_in_the_order_scutil_lists_them() {
    assert_eq!(
        parse_nameservers(SCUTIL_SAMPLE),
        addresses(&["192.168.0.1", "1.1.1.1"])
    );
}

#[test]
fn the_same_nameserver_on_two_resolvers_is_listed_once() {
    let output = "  nameserver[0] : 8.8.8.8\n  nameserver[0] : 8.8.8.8\n  nameserver[1] : 8.8.4.4\n";
    assert_eq!(parse_nameservers(output), addresses(&["8.8.8.8", "8.8.4.4"]));
}

#[test]
fn an_ipv6_nameserver_keeps_its_address_without_the_interface_zone() {
    let output = "  nameserver[0] : fe80::1%en0\n";
    assert_eq!(parse_nameservers(output), addresses(&["fe80::1"]));
}

#[test]
fn lines_that_are_not_nameservers_are_ignored() {
    let output = "  search domain[0] : local\n  reach : 0x00000002 (Reachable)\n  nameserver[0] : not-an-ip\n";
    assert!(parse_nameservers(output).is_empty());
}

#[test]
fn output_without_any_dns_configuration_yields_nothing() {
    assert!(parse_nameservers("No DNS configuration available\n").is_empty());
}

#[test]
fn loopback_nameservers_are_never_blocked() {
    let parsed = parse_nameservers("  nameserver[0] : 127.0.0.1\n  nameserver[1] : ::1\n  nameserver[2] : 192.168.0.1\n");
    assert_eq!(parsed.len(), 3);
    assert_eq!(blockable(&parsed), addresses(&["192.168.0.1"]));
}

#[test]
fn a_router_on_a_private_range_stays_blockable() {
    let parsed = parse_nameservers("  nameserver[0] : 192.168.0.1\n");
    assert_eq!(blockable(&parsed), addresses(&["192.168.0.1"]));
}

#[test]
fn each_nameserver_gets_tcp_and_udp_443_dropped() {
    let anchor = render_anchor(&addresses(&["192.168.0.1"]));

    assert!(anchor.starts_with(HEADER));
    assert!(anchor.contains("block drop out quick proto tcp from any to 192.168.0.1 port 443\n"));
    assert!(anchor.contains("block drop out quick proto udp from any to 192.168.0.1 port 443\n"));
    assert_eq!(anchor.matches("block drop").count(), 2);
}

#[test]
fn an_ipv6_nameserver_is_written_bare_as_pf_expects() {
    let anchor = render_anchor(&addresses(&["2606:4700:4700::1111"]));
    assert!(anchor.contains("to 2606:4700:4700::1111 port 443\n"));
}

#[test]
fn no_nameserver_renders_an_empty_anchor() {
    assert_eq!(render_anchor(&[]), "");
}

#[test]
fn rendering_is_stable_for_the_same_nameservers() {
    let targets = addresses(&["192.168.0.1", "1.1.1.1"]);
    assert_eq!(render_anchor(&targets), render_anchor(&targets));
}

const RESOLV_CONF_SAMPLE: &str = "#
# macOS Notice
#
# This file is not consulted for DNS hostname resolution.
#
nameserver 1.0.0.3
nameserver 1.1.1.3
";

#[test]
fn resolv_conf_nameservers_are_read_past_the_notice_comments() {
    assert_eq!(
        break_enforcer::pf::resolvers::parse_resolv_conf(RESOLV_CONF_SAMPLE),
        addresses(&["1.0.0.3", "1.1.1.3"])
    );
}

#[test]
fn a_resolv_conf_without_nameservers_yields_nothing() {
    assert!(break_enforcer::pf::resolvers::parse_resolv_conf("search local\n").is_empty());
}

#[test]
fn a_cloudflare_resolver_is_blocked_on_both_protocols() {
    let anchor = render_anchor(&blockable(&addresses(&["1.1.1.3", "1.0.0.3"])));

    assert!(anchor.contains("proto tcp from any to 1.1.1.3 port 443\n"));
    assert!(anchor.contains("proto udp from any to 1.0.0.3 port 443\n"));
    assert_eq!(anchor.matches("block drop").count(), 4);
}
