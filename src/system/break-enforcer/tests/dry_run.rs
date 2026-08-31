use break_core::rules::SiteTarget;
use break_enforcer::{is_dry_run, platform_enforcer, Enforcer, DRY_RUN_ENV};

#[test]
fn dry_run_skips_enforcement_and_succeeds() {
    unsafe { std::env::set_var(DRY_RUN_ENV, "1") };
    assert!(is_dry_run());

    let enforcer = platform_enforcer();
    let sites = vec![SiteTarget {
        host: "youtube.com".to_string(),
    }];
    assert!(enforcer.apply_sites(&sites).is_ok());
    assert!(enforcer.clear_sites().is_ok());

    unsafe { std::env::remove_var(DRY_RUN_ENV) };
    assert!(!is_dry_run());
}
