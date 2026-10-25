import { useState } from "react";

import type { InstantBlock, SiteGroup, TimeWindow } from "@/shared/types";
import { WEEKDAYS } from "@/shared/types";
import { Button } from "@/shared/ui/Button";
import { EmptyState } from "@/shared/ui/EmptyState";
import { PlusIcon } from "@/shared/ui/icons";

import type { BlockingSession } from "./active";
import { formatClock } from "./active";
import type { WindowDraft } from "./AddWindowSheet";
import { AddWindowSheet } from "./AddWindowSheet";
import type { InstantSheetMode } from "./InstantSheet";
import { InstantSheet } from "./InstantSheet";
import { RunningBanner } from "./RunningBanner";
import { WeekTimeline } from "./WeekTimeline";
import { WindowRow } from "./WindowRow";

const EXTEND_DEFAULT_MINUTES = 60;

type OpenSheet =
  | { kind: "none" }
  | { kind: "window"; draft?: WindowDraft; extending: boolean }
  | { kind: "instant"; mode: InstantSheetMode };

interface ScheduleTabProps {
  windows: TimeWindow[];
  instant: InstantBlock | null;
  groups: SiteGroup[];
  session: BlockingSession | null;
  onAdd: (window: TimeWindow) => void;
  onInstant: (endsAt: Date, groupIds: string[]) => void;
  onToggle: (id: string, next: boolean) => void;
  onRemove: (id: string) => void;
}

function extensionSheet(
  session: BlockingSession,
  groups: SiteGroup[],
): OpenSheet {
  const endsAt = session.endsAt;
  if (endsAt === null) {
    return { kind: "none" };
  }
  if (
    session.instantEndsAt !== null &&
    session.instantEndsAt.getTime() === endsAt.getTime()
  ) {
    return { kind: "instant", mode: { kind: "extend", from: endsAt } };
  }

  const until = new Date(endsAt.getTime() + EXTEND_DEFAULT_MINUTES * 60_000);
  const known = groups.map((group) => group.id);
  return {
    kind: "window",
    extending: true,
    draft: {
      start: formatClock(endsAt),
      end: formatClock(until),
      days: [WEEKDAYS[(endsAt.getDay() + 6) % 7]],
      groupIds: session.groupIds.filter((id) => known.includes(id)),
    },
  };
}

export function ScheduleTab({
  windows,
  instant,
  groups,
  session,
  onAdd,
  onInstant,
  onToggle,
  onRemove,
}: ScheduleTabProps) {
  const [sheet, setSheet] = useState<OpenSheet>({ kind: "none" });
  const running = session?.windowIds ?? [];

  function close() {
    setSheet({ kind: "none" });
  }

  function handleAdd(window: TimeWindow) {
    onAdd(window);
    close();
  }

  function handleInstant(endsAt: Date, groupIds: string[]) {
    onInstant(endsAt, groupIds);
    close();
  }

  return (
    <>
      {session !== null && (
        <RunningBanner
          session={session}
          onExtend={() => setSheet(extensionSheet(session, groups))}
        />
      )}

      <WeekTimeline windows={windows} instant={instant} />

      <div className="mt-5 mb-3 flex items-center justify-between">
        <div className="flex items-baseline gap-2.5">
          <span className="text-[13px] font-bold text-ink">시간대</span>
          <span className="text-xs text-ink-faint">
            {windows.filter((window) => window.enabled).length} /{" "}
            {windows.length} 켜짐
          </span>
        </div>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            onClick={() =>
              setSheet({ kind: "instant", mode: { kind: "start" } })
            }
          >
            지금부터 막기
          </Button>
          <Button
            onClick={() => setSheet({ kind: "window", extending: false })}
          >
            <PlusIcon className="size-3" />
            시간대 추가
          </Button>
        </div>
      </div>

      {windows.length === 0 ? (
        <EmptyState
          title="아직 시간대가 없습니다"
          hint="시간대를 만든 뒤 켜야 차단이 시작됩니다"
        />
      ) : (
        <ul className="flex list-none flex-col gap-2 p-0">
          {windows.map((window) => (
            <WindowRow
              key={window.id}
              window={window}
              groups={groups}
              running={running.includes(window.id)}
              onToggle={onToggle}
              onRemove={onRemove}
            />
          ))}
        </ul>
      )}

      {sheet.kind === "window" && (
        <AddWindowSheet
          groups={groups}
          draft={sheet.draft}
          extending={sheet.extending}
          onAdd={handleAdd}
          onClose={close}
        />
      )}

      {sheet.kind === "instant" && (
        <InstantSheet
          mode={sheet.mode}
          groups={groups}
          onConfirm={handleInstant}
          onClose={close}
        />
      )}
    </>
  );
}
