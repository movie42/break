import type { Rules } from "./rules";

export type DaemonStatus =
  | { kind: "notInstalled" }
  | { kind: "installed" }
  | { kind: "running" };

export interface AppState {
  rules: Rules;
  blockingNow: boolean;
  daemon: DaemonStatus;
  rulesPath: string;
  rejectedSites: string[];
}
