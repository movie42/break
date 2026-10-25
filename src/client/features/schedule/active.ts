import type { InstantBlock, TimeWindow } from "@/shared/types";

import { MINUTES_PER_WEEK, mergedByDay, mergedWeek } from "./merge";
import { MINUTES_PER_DAY } from "./time";

export interface BlockingSession {
  windowIds: string[];
  groupIds: string[];
  endsAt: Date | null;
  instantEndsAt: Date | null;
}

const MAX_CHAIN = 16;

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

export function instantEndsAt(
  instant: InstantBlock | null,
  at: Date,
): Date | null {
  if (instant === null) {
    return null;
  }
  const ends = new Date(instant.endsAt);
  return ends.getTime() > at.getTime() ? ends : null;
}

function windowsEndAt(enabled: TimeWindow[], at: Date): Date | null {
  const left = minutesLeft(enabled, at);
  if (left === null) {
    return null;
  }
  const startOfMinute = new Date(at);
  startOfMinute.setSeconds(0, 0);
  return new Date(startOfMinute.getTime() + left * 60_000);
}

function releaseAt(
  enabled: TimeWindow[],
  instant: InstantBlock | null,
  at: Date,
): Date | null {
  const coveredAllWeek = mergedWeek(enabled).some(
    (span) => span.end - span.start >= MINUTES_PER_WEEK,
  );
  if (coveredAllWeek) {
    return null;
  }

  let cursor = at;
  for (let step = 0; step < MAX_CHAIN; step++) {
    const candidates = [
      windowsEndAt(enabled, cursor),
      instantEndsAt(instant, cursor),
    ].filter((value): value is Date => value !== null);
    if (candidates.length === 0) {
      break;
    }
    const next = new Date(
      Math.max(...candidates.map((value) => value.getTime())),
    );
    if (next.getTime() <= cursor.getTime()) {
      break;
    }
    cursor = next;
  }
  return cursor;
}

export function currentSession(
  windows: TimeWindow[],
  instant: InstantBlock | null,
  at: Date,
): BlockingSession | null {
  const running = windows.filter((window) => isActiveAt(window, at));
  const instantEnd = instantEndsAt(instant, at);
  if (running.length === 0 && instantEnd === null) {
    return null;
  }

  const sources = running.map((window) => window.groupIds);
  if (instantEnd !== null && instant !== null) {
    sources.push(instant.groupIds);
  }

  const groupIds: string[] = [];
  for (const ids of sources) {
    for (const id of ids) {
      if (!groupIds.includes(id)) {
        groupIds.push(id);
      }
    }
  }

  return {
    windowIds: running.map((window) => window.id),
    groupIds,
    endsAt: releaseAt(
      windows.filter((window) => window.enabled),
      instant,
      at,
    ),
    instantEndsAt: instantEnd,
  };
}

export function formatCountdown(from: Date, to: Date): string {
  const total = Math.max(0, Math.ceil((to.getTime() - from.getTime()) / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");

  if (hours === 0) {
    return `${minutes}:${seconds}`;
  }
  return `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`;
}

export function formatClock(at: Date): string {
  return `${String(at.getHours()).padStart(2, "0")}:${String(at.getMinutes()).padStart(2, "0")}`;
}

export function formatReleaseAt(at: Date, to: Date): string {
  const time = formatClock(to);
  const sameDay = at.toDateString() === to.toDateString();
  return sameDay ? time : `${to.getMonth() + 1}월 ${to.getDate()}일 ${time}`;
}
