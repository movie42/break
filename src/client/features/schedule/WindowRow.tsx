import type { SiteGroup, TimeWindow } from "@/shared/types";
import { WEEKDAY_LABELS, WEEKDAYS } from "@/shared/types";
import { CloseIcon, LockIcon } from "@/shared/ui/icons";
import { IconButton } from "@/shared/ui/IconButton";
import { Switch } from "@/shared/ui/Switch";

import { crossesMidnight, isAllDay, toInputValue } from "./time";

interface WindowRowProps {
  window: TimeWindow;
  groups: SiteGroup[];
  running: boolean;
  onToggle: (id: string, next: boolean) => void;
  onRemove: (id: string) => void;
}

function describeTime(window: TimeWindow): string {
  if (isAllDay(window.start, window.end)) {
    return "하루 종일";
  }
  return `${toInputValue(window.start)} — ${toInputValue(window.end)}`;
}

function describeNote(window: TimeWindow): string {
  if (isAllDay(window.start, window.end)) {
    return "00:00부터 24:00까지";
  }
  if (crossesMidnight(window.start, window.end)) {
    return `다음 날 ${toInputValue(window.end)}까지`;
  }
  return "같은 날 안에서";
}

export function WindowRow({
  window,
  groups,
  running,
  onToggle,
  onRemove,
}: WindowRowProps) {
  const linked = window.groupIds
    .map((id) => groups.find((group) => group.id === id))
    .filter((group): group is SiteGroup => group !== undefined);

  return (
    <li
      className={`group flex items-center gap-4.5 rounded-xl border bg-surface px-4 py-3 shadow-card transition-colors ${
        running ? "border-accent-line" : "border-line hover:border-line-strong"
      }`}
    >
      <Switch
        checked={window.enabled}
        disabled={running}
        label={
          running
            ? "실행 중에는 끌 수 없습니다"
            : window.enabled
              ? "이 시간대 끄기"
              : "이 시간대 켜기"
        }
        onChange={(next) => onToggle(window.id, next)}
      />

      <div className="flex w-40 shrink-0 flex-col gap-1">
        <span
          className={`text-[21px] leading-tight font-bold tracking-tight ${
            window.enabled ? "text-ink" : "text-ink-faint"
          }`}
        >
          {describeTime(window)}
        </span>
        <span className="text-[11px] text-ink-faint">
          {window.enabled ? describeNote(window) : "꺼짐 — 실행하지 않습니다"}
        </span>
      </div>

      <div className="flex shrink-0 items-center gap-1.5">
        {WEEKDAYS.map((day) => {
          const on = window.days.includes(day);
          return (
            <span
              key={day}
              className={`flex size-[21px] items-center justify-center rounded-full text-[10.5px] font-semibold ${
                on
                  ? window.enabled
                    ? "bg-accent-soft text-accent"
                    : "bg-surface-muted text-ink-faint"
                  : "text-ink-ghost"
              }`}
            >
              {WEEKDAY_LABELS[day]}
            </span>
          );
        })}
      </div>

      <div className="flex min-w-0 flex-1 flex-wrap gap-1.5">
        {linked.length === 0 ? (
          <span className="rounded-full border border-dashed border-line-strong px-2.5 py-0.5 text-[11px] text-ink-faint">
            그룹 없음
          </span>
        ) : (
          linked.map((group) => (
            <span
              key={group.id}
              className="rounded-full border border-line px-2.5 py-0.5 text-[11px] whitespace-nowrap text-ink-muted"
            >
              {group.name}
            </span>
          ))
        )}
      </div>

      {running ? (
        <span
          title="실행 중에는 지울 수 없습니다"
          className="flex size-7 shrink-0 items-center justify-center text-accent"
        >
          <LockIcon className="size-[15px]" />
        </span>
      ) : (
        <IconButton
          label="시간대 삭제"
          onClick={() => onRemove(window.id)}
          className="shrink-0 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
        >
          <CloseIcon className="size-[15px]" />
        </IconButton>
      )}
    </li>
  );
}
