import { useState } from "react";

import { Button } from "@/shared/ui/Button";
import { EmptyState } from "@/shared/ui/EmptyState";
import type { TimeWindow, Weekday } from "@/shared/types";
import { WEEKDAYS, WEEKDAY_LABELS } from "@/shared/types";

import { crossesMidnight, fromInputValue, isAllDay, toInputValue } from "./time";

interface ScheduleEditorProps {
  windows: TimeWindow[];
  onAdd: (window: TimeWindow) => void;
  onRemove: (id: string) => void;
  error: string | null;
}

function describe(window: TimeWindow): string {
  if (isAllDay(window.start, window.end)) {
    return "하루 종일";
  }
  const range = `${toInputValue(window.start)} ~ ${toInputValue(window.end)}`;
  return crossesMidnight(window.start, window.end)
    ? `${range} (다음 날까지)`
    : range;
}

export function ScheduleEditor({
  windows,
  onAdd,
  onRemove,
  error,
}: ScheduleEditorProps) {
  const [start, setStart] = useState("22:00");
  const [end, setEnd] = useState("02:00");
  const [days, setDays] = useState<Weekday[]>([]);

  function toggleDay(day: Weekday) {
    setDays((current) =>
      current.includes(day)
        ? current.filter((item) => item !== day)
        : [...current, day],
    );
  }

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const parsedStart = fromInputValue(start);
    const parsedEnd = fromInputValue(end);
    if (!parsedStart || !parsedEnd) {
      return;
    }
    onAdd({
      id: crypto.randomUUID(),
      start: parsedStart,
      end: parsedEnd,
      days: WEEKDAYS.filter((day) => days.includes(day)),
    });
    setDays([]);
  }

  return (
    <section className="flex flex-col gap-4">
      <form
        onSubmit={handleSubmit}
        className="flex flex-col gap-4 rounded-lg border border-line bg-surface p-4"
      >
        <fieldset className="flex flex-col gap-2">
          <legend className="text-sm font-medium text-ink">요일</legend>
          <div className="flex flex-wrap gap-2">
            {WEEKDAYS.map((day) => {
              const selected = days.includes(day);
              return (
                <button
                  key={day}
                  type="button"
                  aria-pressed={selected}
                  onClick={() => toggleDay(day)}
                  className={`h-9 w-9 rounded-full border text-sm transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
                    selected
                      ? "border-accent bg-accent text-accent-fg"
                      : "border-line bg-surface text-ink-muted hover:bg-surface-muted"
                  }`}
                >
                  {WEEKDAY_LABELS[day]}
                </button>
              );
            })}
          </div>
        </fieldset>

        <div className="flex flex-wrap items-end gap-3">
          <label className="flex flex-col gap-1 text-sm text-ink">
            시작
            <input
              type="time"
              value={start}
              onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                setStart(event.target.value)
              }
              className="rounded-md border border-line bg-surface px-3 py-2 text-sm text-ink focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
            />
          </label>
          <label className="flex flex-col gap-1 text-sm text-ink">
            종료
            <input
              type="time"
              value={end}
              onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                setEnd(event.target.value)
              }
              className="rounded-md border border-line bg-surface px-3 py-2 text-sm text-ink focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
            />
          </label>
          <Button type="submit" disabled={days.length === 0}>
            추가
          </Button>
        </div>

        <p className="text-xs text-ink-muted">
          종료가 시작보다 이르면 자정을 넘는 구간으로 봅니다. 선택한 요일에
          시작해서 다음 날 종료 시각까지 이어집니다.
        </p>
      </form>

      {error && <p className="text-sm text-danger">{error}</p>}

      {windows.length === 0 ? (
        <EmptyState message="아직 추가한 시간대가 없습니다." />
      ) : (
        <ul className="divide-y divide-line overflow-hidden rounded-lg border border-line bg-surface">
          {windows.map((window) => (
            <li
              key={window.id}
              className="flex items-center justify-between gap-4 px-4 py-3"
            >
              <div className="flex flex-col gap-1">
                <span className="text-sm text-ink">{describe(window)}</span>
                <span className="text-xs text-ink-muted">
                  {window.days.map((day) => WEEKDAY_LABELS[day]).join(" · ")}
                </span>
              </div>
              <Button variant="danger" onClick={() => onRemove(window.id)}>
                삭제
              </Button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
