import type { TimeOfDay } from "@/shared/types";

export const MINUTES_PER_DAY = 24 * 60;

export function toInputValue(time: TimeOfDay): string {
  const hour = String(time.hour).padStart(2, "0");
  const minute = String(time.minute).padStart(2, "0");
  return `${hour}:${minute}`;
}

export function fromInputValue(value: string): TimeOfDay | null {
  const match = /^(\d{2}):(\d{2})$/.exec(value);
  if (!match) {
    return null;
  }
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (hour > 23 || minute > 59) {
    return null;
  }
  return { hour, minute };
}

export function toMinutes(time: TimeOfDay): number {
  return time.hour * 60 + time.minute;
}

export function formatMinutes(minutes: number): string {
  const capped = Math.min(minutes, MINUTES_PER_DAY);
  const hour = Math.floor(capped / 60);
  const minute = capped % 60;
  return `${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}`;
}

export function crossesMidnight(start: TimeOfDay, end: TimeOfDay): boolean {
  return toMinutes(start) > toMinutes(end);
}

export function isAllDay(start: TimeOfDay, end: TimeOfDay): boolean {
  return toMinutes(start) === toMinutes(end);
}
