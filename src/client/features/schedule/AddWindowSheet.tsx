import { useState } from "react";

import type { SiteGroup, TimeWindow, Weekday } from "@/shared/types";
import { WEEKDAY_LABELS, WEEKDAY_PRESETS, WEEKDAYS } from "@/shared/types";
import { SectionLabel } from "@/shared/ui/SectionLabel";
import { Sheet } from "@/shared/ui/Sheet";
import { ToggleChip } from "@/shared/ui/ToggleChip";

import { fromInputValue } from "./time";

const WEEKDAY_DEFAULT: Weekday[] = [
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
];

const INPUT_CLASS =
  "h-10 rounded-lg border border-line-strong bg-surface px-3 text-[15px] font-medium text-ink outline-none focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent";

interface AddWindowSheetProps {
  groups: SiteGroup[];
  onAdd: (window: TimeWindow) => void;
  onClose: () => void;
}

export function AddWindowSheet({
  groups,
  onAdd,
  onClose,
}: AddWindowSheetProps) {
  const [days, setDays] = useState<Weekday[]>(WEEKDAY_DEFAULT);
  const [start, setStart] = useState("22:00");
  const [end, setEnd] = useState("02:00");
  const [groupIds, setGroupIds] = useState<string[]>([]);

  const parsedStart = fromInputValue(start);
  const parsedEnd = fromInputValue(end);
  const valid =
    days.length > 0 &&
    groupIds.length > 0 &&
    parsedStart !== null &&
    parsedEnd !== null;

  const hint =
    parsedStart === null || parsedEnd === null
      ? "시간을 다시 확인하세요"
      : start === end
        ? "하루 종일"
        : end < start
          ? "다음 날까지 이어집니다"
          : "";

  function toggleDay(day: Weekday) {
    setDays((current) =>
      current.includes(day)
        ? current.filter((item) => item !== day)
        : [...current, day],
    );
  }

  function toggleGroup(id: string) {
    setGroupIds((current) =>
      current.includes(id)
        ? current.filter((item) => item !== id)
        : [...current, id],
    );
  }

  function handleConfirm() {
    if (parsedStart === null || parsedEnd === null) {
      return;
    }
    onAdd({
      id: crypto.randomUUID(),
      start: parsedStart,
      end: parsedEnd,
      days: WEEKDAYS.filter((day) => days.includes(day)),
      groupIds: groups
        .filter((group) => groupIds.includes(group.id))
        .map((group) => group.id),
      enabled: false,
    });
  }

  return (
    <Sheet
      title="시간대 추가"
      width="wide"
      footerNote={
        valid
          ? "만든 뒤 목록에서 켜야 실행됩니다"
          : "요일과 그룹을 하나 이상 고르세요"
      }
      confirmLabel="추가"
      confirmDisabled={!valid}
      onConfirm={handleConfirm}
      onClose={onClose}
    >
      <div className="mb-2 flex items-center justify-between">
        <SectionLabel>요일</SectionLabel>
        <div className="flex gap-1.5">
          {WEEKDAY_PRESETS.map((preset) => (
            <button
              key={preset.label}
              type="button"
              onClick={() => setDays(preset.days)}
              className="rounded-full border border-line px-2.5 py-0.5 text-[11px] text-ink-muted transition-colors hover:bg-surface-muted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {preset.label}
            </button>
          ))}
        </div>
      </div>
      <div className="mb-5 flex gap-1.5">
        {WEEKDAYS.map((day) => (
          <ToggleChip
            key={day}
            shape="circle"
            selected={days.includes(day)}
            onClick={() => toggleDay(day)}
          >
            {WEEKDAY_LABELS[day]}
          </ToggleChip>
        ))}
      </div>

      <div className="mb-2">
        <SectionLabel>시간</SectionLabel>
      </div>
      <div className="mb-5 flex items-center gap-3">
        <input
          type="time"
          value={start}
          aria-label="시작"
          onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
            setStart(event.target.value)
          }
          className={INPUT_CLASS}
        />
        <span className="text-[13px] text-ink-faint">—</span>
        <input
          type="time"
          value={end}
          aria-label="종료"
          onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
            setEnd(event.target.value)
          }
          className={INPUT_CLASS}
        />
        <span className="text-[11.5px] text-ink-faint">{hint}</span>
      </div>

      <div className="mb-2">
        <SectionLabel>차단할 그룹</SectionLabel>
      </div>
      <div className="mb-5 flex flex-wrap gap-1.5">
        {groups.length === 0 ? (
          <span className="text-[11.5px] text-ink-faint">
            사이트 탭에서 그룹을 먼저 만드세요.
          </span>
        ) : (
          groups.map((group) => (
            <ToggleChip
              key={group.id}
              shape="pill"
              selected={groupIds.includes(group.id)}
              onClick={() => toggleGroup(group.id)}
            >
              {group.name}
            </ToggleChip>
          ))
        )}
      </div>
    </Sheet>
  );
}
