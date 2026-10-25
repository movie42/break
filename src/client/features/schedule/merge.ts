import type { TimeWindow, Weekday } from "@/shared/types";
import { WEEKDAYS } from "@/shared/types";

import { MINUTES_PER_DAY, toMinutes } from "./time";

export const MINUTES_PER_WEEK = MINUTES_PER_DAY * WEEKDAYS.length;

export interface Span {
  start: number;
  end: number;
}

function dayIndex(day: Weekday): number {
  return WEEKDAYS.indexOf(day);
}

function spansOf(window: TimeWindow): { day: number; span: Span }[] {
  const start = toMinutes(window.start);
  const end = toMinutes(window.end);
  const out: { day: number; span: Span }[] = [];

  for (const day of window.days) {
    const index = dayIndex(day);
    if (index < 0) {
      continue;
    }
    if (start === end) {
      out.push({ day: index, span: { start: 0, end: MINUTES_PER_DAY } });
    } else if (end > start) {
      out.push({ day: index, span: { start, end } });
    } else {
      out.push({ day: index, span: { start, end: MINUTES_PER_DAY } });
      out.push({ day: (index + 1) % WEEKDAYS.length, span: { start: 0, end } });
    }
  }

  return out;
}

function merge(spans: Span[]): Span[] {
  const sorted = [...spans].sort((left, right) => left.start - right.start);
  const out: Span[] = [];

  for (const span of sorted) {
    const last = out[out.length - 1];
    if (last && span.start <= last.end) {
      last.end = Math.max(last.end, span.end);
    } else {
      out.push({ ...span });
    }
  }

  return out;
}

export function mergedByDay(windows: TimeWindow[]): Span[][] {
  const byDay: Span[][] = WEEKDAYS.map(() => []);

  for (const window of windows) {
    for (const { day, span } of spansOf(window)) {
      byDay[day].push(span);
    }
  }

  return byDay.map(merge);
}

export function mergedWeek(windows: TimeWindow[]): Span[] {
  const flat: Span[] = [];

  mergedByDay(windows).forEach((spans, day) => {
    for (const span of spans) {
      flat.push({
        start: day * MINUTES_PER_DAY + span.start,
        end: day * MINUTES_PER_DAY + span.end,
      });
    }
  });

  const merged = merge(flat);
  if (merged.length < 2) {
    return merged;
  }

  const first = merged[0];
  const last = merged[merged.length - 1];
  if (first.start === 0 && last.end === MINUTES_PER_WEEK) {
    return [
      ...merged.slice(1, -1),
      { start: last.start, end: MINUTES_PER_WEEK + first.end },
    ];
  }

  return merged;
}
