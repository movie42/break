import { invoke } from "@tauri-apps/api/core";

import type { AppState, DaemonStatus, QuitReport } from "../types/app-state";
import type { Rules } from "../types/rules";

export function loadRules(): Promise<AppState> {
  return invoke<AppState>("load_rules");
}

export function saveRules(rules: Rules): Promise<AppState> {
  return invoke<AppState>("save_rules", { rules });
}

export function daemonStatus(): Promise<DaemonStatus> {
  return invoke<DaemonStatus>("daemon_status");
}

export function installDaemon(): Promise<AppState> {
  return invoke<AppState>("install_daemon");
}

export function uninstallDaemon(): Promise<AppState> {
  return invoke<AppState>("uninstall_daemon");
}

export function runningBrowsers(): Promise<string[]> {
  return invoke<string[]>("running_browsers");
}

export function quitBrowsers(): Promise<QuitReport> {
  return invoke<QuitReport>("quit_browsers");
}
