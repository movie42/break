import { useState } from "react";

import { useNow } from "@/shared/hooks/useNow";
import type { SiteGroup } from "@/shared/types";
import { SectionLabel } from "@/shared/ui/SectionLabel";
import { Sheet } from "@/shared/ui/Sheet";
import { ToggleChip } from "@/shared/ui/ToggleChip";

import { formatReleaseAt } from "./active";

const PRESET_MINUTES: readonly number[] = import.meta.env.DEV
  ? [1, 25, 60, 120, 240]
  : [25, 60, 120, 240];
const PREVIEW_MS = 1000;
const MAX_HOURS = 23;

const INPUT_CLASS =
  "h-10 w-16 rounded-lg border border-line-strong bg-surface px-3 text-[15px] font-medium text-ink outline-none focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent";

export type InstantSheetMode =
  { kind: "start" } | { kind: "extend"; from: Date };

interface InstantSheetProps {
  mode: InstantSheetMode;
  groups: SiteGroup[];
  onConfirm: (endsAt: Date, groupIds: string[]) => void;
  onClose: () => void;
}

function presetLabel(minutes: number): string {
  return minutes < 60 ? `${minutes}분` : `${minutes / 60}시간`;
}

function parseCount(value: string, max: number): number | null {
  if (!/^\d{1,2}$/.test(value)) {
    return null;
  }
  const count = Number(value);
  return count <= max ? count : null;
}

export function InstantSheet({
  mode,
  groups,
  onConfirm,
  onClose,
}: InstantSheetProps) {
  const now = useNow(PREVIEW_MS);
  const [hours, setHours] = useState("1");
  const [minutes, setMinutes] = useState("0");
  const [groupIds, setGroupIds] = useState<string[]>([]);

  const parsedHours = parseCount(hours, MAX_HOURS);
  const parsedMinutes = parseCount(minutes, 59);
  const length =
    parsedHours === null || parsedMinutes === null
      ? null
      : parsedHours * 60 + parsedMinutes;

  const base = mode.kind === "extend" ? mode.from : now;
  const endsAt =
    length === null ? null : new Date(base.getTime() + length * 60_000);
  const needsGroups = mode.kind === "start";
  const valid =
    length !== null && length > 0 && (!needsGroups || groupIds.length > 0);

  function pickPreset(total: number) {
    setHours(String(Math.floor(total / 60)));
    setMinutes(String(total % 60));
  }

  function toggleGroup(id: string) {
    setGroupIds((current) =>
      current.includes(id)
        ? current.filter((item) => item !== id)
        : [...current, id],
    );
  }

  function handleConfirm() {
    if (endsAt === null || !valid) {
      return;
    }
    onConfirm(
      endsAt,
      groups
        .filter((group) => groupIds.includes(group.id))
        .map((group) => group.id),
    );
  }

  const footerNote = !valid
    ? needsGroups
      ? "길이와 그룹을 고르세요"
      : "늘릴 길이를 고르세요"
    : `${formatReleaseAt(now, endsAt ?? now)}까지 · 끌 수 없음`;

  return (
    <Sheet
      title={mode.kind === "start" ? "지금부터 막기" : "즉시 차단 연장"}
      width="wide"
      footerNote={footerNote}
      confirmLabel={mode.kind === "start" ? "시작" : "연장"}
      confirmDisabled={!valid}
      onConfirm={handleConfirm}
      onClose={onClose}
    >
      <div className="mb-2">
        <SectionLabel>
          {mode.kind === "start" ? "길이" : "늘릴 길이"}
        </SectionLabel>
      </div>
      <div className="mb-3 flex gap-1.5">
        {PRESET_MINUTES.map((total) => (
          <ToggleChip
            key={total}
            shape="pill"
            selected={length === total}
            onClick={() => pickPreset(total)}
          >
            {presetLabel(total)}
          </ToggleChip>
        ))}
      </div>
      <div className="mb-5 flex items-center gap-2 text-[13px] text-ink-muted">
        <span>직접</span>
        <input
          inputMode="numeric"
          value={hours}
          aria-label="시간"
          onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
            setHours(event.target.value)
          }
          className={INPUT_CLASS}
        />
        <span>시간</span>
        <input
          inputMode="numeric"
          value={minutes}
          aria-label="분"
          onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
            setMinutes(event.target.value)
          }
          className={INPUT_CLASS}
        />
        <span>분</span>
      </div>

      {needsGroups && (
        <>
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
        </>
      )}
    </Sheet>
  );
}
