import type { TimeWindow } from "@/shared/types";

import { MINUTES_PER_WEEK, mergedByDay, mergedWeek } from "./merge";
import { MINUTES_PER_DAY } from "./time";

export interface BlockingSession {
  windowIds: string[];
  groupIds: string[];
  endsAt: Date | null;
}

function weekMinutes(at: Date): number {
  const day = (at.getDay() + 6) % 7;
  return day * MINUTES_PER_DAY + at.getHours() * 60 + at.getMinutes();
}

export function isActiveAt(window: TimeWindow, at: Date): boolean {
  if (!window.enabled) {
    return false;
  }

  const byDay = mergedByDay([window]);
  const dayIndex = (at.getDay() + 6) % 7;
  const minutes = at.getHours() * 60 + at.getMinutes();

  return byDay[dayIndex].some(
    (span) => minutes >= span.start && minutes < span.end,
  );
}

export function coversNow(window: TimeWindow, at: Date): boolean {
  return isActiveAt({ ...window, enabled: true }, at);
}

function minutesLeft(windows: TimeWindow[], at: Date): number | null {
  const now = weekMinutes(at);

  for (const span of mergedWeek(windows)) {
    const shifted = span.end > MINUTES_PER_WEEK ? now + MINUTES_PER_WEEK : now;
    const inside =
      (now >= span.start && now < span.end) ||
      (shifted >= span.start && shifted < span.end);
    if (!inside) {
      continue;
    }
    const from = now >= span.start && now < span.end ? now : shifted;
    return span.end - from;
  }

  return null;
}

export function currentSession(
  windows: TimeWindow[],
  at: Date,
): BlockingSession | null {
  const running = windows.filter((window) => isActiveAt(window, at));
  if (running.length === 0) {
    return null;
  }

  const enabled = windows.filter((window) => window.enabled);
  const left = minutesLeft(enabled, at);
  const covered = mergedWeek(enabled).some(
    (span) => span.end - span.start >= MINUTES_PER_WEEK,
  );

  const startOfMinute = new Date(at);
  startOfMinute.setSeconds(0, 0);

  const groupIds: string[] = [];
  for (const window of running) {
    for (const id of window.groupIds) {
      if (!groupIds.includes(id)) {
        groupIds.push(id);
      }
    }
  }

  return {
    windowIds: running.map((window) => window.id),
    groupIds,
    endsAt:
      covered || left === null
        ? null
        : new Date(startOfMinute.getTime() + left * 60_000),
  };
}

export function formatRemaining(from: Date, to: Date): string {
  const minutes = Math.max(
    0,
    Math.ceil((to.getTime() - from.getTime()) / 60_000),
  );
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;

  if (hours === 0) {
    return `${rest}분`;
  }
  if (rest === 0) {
    return `${hours}시간`;
  }
  return `${hours}시간 ${rest}분`;
}

export function formatReleaseAt(at: Date, to: Date): string {
  const time = `${String(to.getHours()).padStart(2, "0")}:${String(to.getMinutes()).padStart(2, "0")}`;
  const sameDay = at.toDateString() === to.toDateString();
  return sameDay ? time : `${to.getMonth() + 1}월 ${to.getDate()}일 ${time}`;
}
