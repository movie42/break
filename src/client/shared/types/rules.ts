export const RULES_VERSION = 1;

export type Weekday =
  | "monday"
  | "tuesday"
  | "wednesday"
  | "thursday"
  | "friday"
  | "saturday"
  | "sunday";

export const WEEKDAYS: readonly Weekday[] = [
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
  "saturday",
  "sunday",
];

export const WEEKDAY_LABELS: Record<Weekday, string> = {
  monday: "월",
  tuesday: "화",
  wednesday: "수",
  thursday: "목",
  friday: "금",
  saturday: "토",
  sunday: "일",
};

export interface TimeOfDay {
  hour: number;
  minute: number;
}

export interface TimeWindow {
  id: string;
  start: TimeOfDay;
  end: TimeOfDay;
  days: Weekday[];
}

export interface SiteTarget {
  host: string;
}

export interface AppTarget {
  bundleId: string;
  displayName: string;
}

export interface Schedule {
  windows: TimeWindow[];
}

export interface Rules {
  version: number;
  sites: SiteTarget[];
  apps: AppTarget[];
  schedule: Schedule;
}

export const EMPTY_RULES: Rules = {
  version: RULES_VERSION,
  sites: [],
  apps: [],
  schedule: { windows: [] },
};
