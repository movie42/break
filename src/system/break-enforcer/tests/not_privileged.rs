use break_core::rules::{AppTarget, SiteTarget};
use break_enforcer::{platform_enforcer, Enforcer, Error};

fn sites() -> Vec<SiteTarget> {
    vec![SiteTarget {
        host: "youtube.com".to_string(),
    }]
}

fn apps() -> Vec<AppTarget> {
    vec![AppTarget {
        bundle_id: "com.apple.Safari".to_string(),
        display_name: "Safari".to_string(),
    }]
}

#[test]
fn every_method_returns_not_privileged_without_panicking() {
    let enforcer = platform_enforcer();

    assert!(matches!(
        enforcer.apply_sites(&sites()),
        Err(Error::NotPrivileged)
    ));
    assert!(matches!(enforcer.clear_sites(), Err(Error::NotPrivileged)));
    assert!(matches!(
        enforcer.apply_apps(&apps()),
        Err(Error::NotPrivileged)
    ));
    assert!(matches!(enforcer.clear_apps(), Err(Error::NotPrivileged)));
}

#[test]
fn not_privileged_has_a_user_facing_message() {
    assert_eq!(
        Error::NotPrivileged.to_string(),
        "차단을 적용할 권한이 없습니다"
    );
}
