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
  guarded: boolean;
  mailAccount: string | null;
  releaseDelaySeconds: number;
}

export interface MailAccount {
  host: string;
  port: number;
  username: string;
  secret: string;
  from: string;
}

export type EventKind =
  | { kind: "started" }
  | { kind: "stopped" }
  | { kind: "down"; from: string; to: string }
  | { kind: "hostsRestored" }
  | { kind: "rulesChanged" }
  | { kind: "request"; message: string }
  | { kind: "released"; message: string }
  | { kind: "mailFailed"; message: string };

export type BreakEvent = { at: string } & EventKind;

export interface QuitReport {
  closed: string[];
  stillOpen: string[];
}
