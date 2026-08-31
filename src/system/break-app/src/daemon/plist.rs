use std::path::Path;

use super::paths::{installed_binary, LABEL, LOG_PATH};

pub fn render(rules_path: &Path) -> String {
    let binary = installed_binary();
    let arguments = [
        binary.to_string_lossy().into_owned(),
        "--rules".to_string(),
        rules_path.to_string_lossy().into_owned(),
    ];
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
