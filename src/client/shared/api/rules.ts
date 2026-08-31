import { invoke } from "@tauri-apps/api/core";

import type { AppState } from "../types/app-state";
import type { Rules } from "../types/rules";

export function loadRules(): Promise<AppState> {
  return invoke<AppState>("load_rules");
}

export function saveRules(rules: Rules): Promise<AppState> {
  return invoke<AppState>("save_rules", { rules });
}
