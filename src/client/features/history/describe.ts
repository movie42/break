import type { BreakEvent } from "@/shared/types";

export function formatStamp(at: string): string {
  const date = new Date(at);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function clock(at: string): string {
  return formatStamp(at).slice(6);
}

export function describeEvent(event: BreakEvent): string {
  switch (event.kind) {
    case "started":
      return "차단 프로그램 켜짐";
    case "stopped":
      return "차단 프로그램 멈춤";
    case "down":
      return `차단 프로그램이 꺼져 있었음 (${formatStamp(event.from)}~${clock(event.to)})`;
    case "hostsRestored":
      return "hosts 바깥 수정 → 되돌림";
    case "rulesChanged":
      return "규칙이 바뀜";
    case "request":
    case "released":
      return event.message;
    case "mailFailed":
      return `알림 메일을 보내지 못함: ${event.message}`;
  }
}

const WARN_FOR_MS = 7 * 24 * 60 * 60 * 1000;

export function latestDowntime(
  events: BreakEvent[],
  now: Date,
): Extract<BreakEvent, { kind: "down" }> | null {
  for (const event of events) {
    if (event.kind === "down") {
      return now.getTime() - new Date(event.to).getTime() < WARN_FOR_MS
        ? event
        : null;
    }
  }
  return null;
}
