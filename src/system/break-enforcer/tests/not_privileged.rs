use break_core::rules::AppTarget;
use break_enforcer::{platform_enforcer, Enforcer, Error};

fn apps() -> Vec<AppTarget> {
    vec![AppTarget {
        bundle_id: "com.apple.Safari".to_string(),
        display_name: "Safari".to_string(),
    }]
}

#[test]
fn app_blocking_is_not_implemented_yet() {
    let enforcer = platform_enforcer();

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
