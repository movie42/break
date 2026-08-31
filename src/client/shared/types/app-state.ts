import type { Rules } from "./rules";

export type EnforcementStatus =
  | { kind: "applied" }
  | { kind: "notPrivileged"; message: string }
  | { kind: "failed"; message: string };

export interface AppState {
  rules: Rules;
  blockingNow: boolean;
  enforcement: EnforcementStatus;
  rulesPath: string;
  rejectedSites: string[];
}
