import type { TimeOfDay } from "@/shared/types";

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

export function crossesMidnight(start: TimeOfDay, end: TimeOfDay): boolean {
  return start.hour * 60 + start.minute > end.hour * 60 + end.minute;
}

export function isAllDay(start: TimeOfDay, end: TimeOfDay): boolean {
  return start.hour === end.hour && start.minute === end.minute;
}
