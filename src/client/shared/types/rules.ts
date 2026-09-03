export const RULES_VERSION = 3;

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

export const WEEKDAY_PRESETS: readonly { label: string; days: Weekday[] }[] = [
  {
    label: "평일",
    days: ["monday", "tuesday", "wednesday", "thursday", "friday"],
  },
  { label: "주말", days: ["saturday", "sunday"] },
  { label: "매일", days: [...WEEKDAYS] },
];

export interface TimeOfDay {
  hour: number;
  minute: number;
}

export interface TimeWindow {
  id: string;
  start: TimeOfDay;
  end: TimeOfDay;
  days: Weekday[];
  groupIds: string[];
  enabled: boolean;
}

export interface SiteTarget {
  host: string;
}

export interface AppTarget {
  bundleId: string;
  displayName: string;
}

export interface SiteGroup {
  id: string;
  name: string;
  sites: SiteTarget[];
  apps: AppTarget[];
}

export interface Schedule {
  windows: TimeWindow[];
}

export interface Rules {
  version: number;
  groups: SiteGroup[];
  schedule: Schedule;
}

export const EMPTY_RULES: Rules = {
  version: RULES_VERSION,
  groups: [],
  schedule: { windows: [] },
};
