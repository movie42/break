import type { Rules } from "./rules";

export type DaemonStatus =
  { kind: "notInstalled" } | { kind: "installed" } | { kind: "running" };

export type DnsGuardStatus =
  { kind: "off" } | { kind: "applied" } | { kind: "failed" };

export interface AppState {
  rules: Rules;
  blockingNow: boolean;
  daemon: DaemonStatus;
  daemonNeedsReinstall: boolean;
  rulesPath: string;
  rejectedSites: string[];
  dnsGuard: DnsGuardStatus;
}

export interface QuitReport {
  closed: string[];
  stillOpen: string[];
}
