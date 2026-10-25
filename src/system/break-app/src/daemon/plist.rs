use std::path::Path;

use super::paths::{installed_binary, LABEL, LOG_PATH};

pub fn release_delay_seconds() -> i64 {
    if cfg!(debug_assertions) {
        break_core::change::TEST_RELEASE_DELAY_SECONDS
    } else {
        break_core::change::RELEASE_DELAY_SECONDS
    }
}

pub fn render(user_rules_path: &Path) -> String {
    let binary = installed_binary();
    let mut arguments = vec![
        binary.to_string_lossy().into_owned(),
        "--rules".to_string(),
        break_core::store::SYSTEM_RULES_PATH.to_string(),
        "--import".to_string(),
        user_rules_path.to_string_lossy().into_owned(),
        "--client".to_string(),
        std::env::current_exe()
            .map(|exe| exe.to_string_lossy().into_owned())
            .unwrap_or_default(),
    ];
    if cfg!(debug_assertions) {
        arguments.push("--release-delay".to_string());
        arguments.push(release_delay_seconds().to_string());
    }
    let argument_nodes = arguments
        .iter()
        .map(|argument| format!("    <string>{}</string>", escape_xml(argument)))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
{argument_nodes}
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{LOG_PATH}</string>
  <key>StandardErrorPath</key>
  <string>{LOG_PATH}</string>
</dict>
</plist>
"#
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
