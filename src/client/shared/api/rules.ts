import { invoke } from "@tauri-apps/api/core";

import type {
  AppState,
  BreakEvent,
  DaemonStatus,
  MailAccount,
  QuitReport,
} from "../types/app-state";
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

export function setPassword(password: string): Promise<AppState> {
  return invoke<AppState>("set_password", { password });
}

export function consentAdultFilter(): Promise<AppState> {
  return invoke<AppState>("consent_adult_filter");
}

export function requestAdultFilterOff(password: string): Promise<AppState> {
  return invoke<AppState>("request_adult_filter_off", { password });
}

export function cancelAdultFilterOff(): Promise<AppState> {
  return invoke<AppState>("cancel_adult_filter_off");
}

export function enableAdultFilter(): Promise<AppState> {
  return invoke<AppState>("enable_adult_filter");
}

export function addRecipient(email: string): Promise<AppState> {
  return invoke<AppState>("add_recipient", { email });
}

export function requestRecipientRemoval(
  email: string,
  password: string,
): Promise<AppState> {
  return invoke<AppState>("request_recipient_removal", { email, password });
}

export function cancelRecipientRemoval(email: string): Promise<AppState> {
  return invoke<AppState>("cancel_recipient_removal", { email });
}

export function setMailAccount(
  account: MailAccount,
  password: string | null,
): Promise<AppState> {
  return invoke<AppState>("set_mail_account", { account, password });
}

export function recentEvents(): Promise<BreakEvent[]> {
  return invoke<BreakEvent[]>("recent_events");
}

export function quitApp(): Promise<void> {
  return invoke<void>("quit_app");
}
