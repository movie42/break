import { useState } from "react";

import type { SiteGroup, TimeWindow } from "@/shared/types";
import { Button } from "@/shared/ui/Button";
import { EmptyState } from "@/shared/ui/EmptyState";
import { PlusIcon } from "@/shared/ui/icons";

import type { BlockingSession } from "./active";
import { AddWindowSheet } from "./AddWindowSheet";
import { RunningBanner } from "./RunningBanner";
import { WeekTimeline } from "./WeekTimeline";
import { WindowRow } from "./WindowRow";

interface ScheduleTabProps {
  windows: TimeWindow[];
  groups: SiteGroup[];
  session: BlockingSession | null;
  now: Date;
  onAdd: (window: TimeWindow) => void;
  onToggle: (id: string, next: boolean) => void;
  onRemove: (id: string) => void;
}

export function ScheduleTab({
  windows,
  groups,
  session,
  now,
  onAdd,
  onToggle,
  onRemove,
}: ScheduleTabProps) {
  const [adding, setAdding] = useState(false);
  const running = session?.windowIds ?? [];

  function handleAdd(window: TimeWindow) {
    onAdd(window);
    setAdding(false);
  }

  return (
    <>
      {session !== null && <RunningBanner session={session} now={now} />}

      <WeekTimeline windows={windows} now={now} />

      <div className="mt-5 mb-3 flex items-center justify-between">
        <div className="flex items-baseline gap-2.5">
          <span className="text-[13px] font-bold text-ink">시간대</span>
          <span className="text-xs text-ink-faint">
            {windows.filter((window) => window.enabled).length} /{" "}
            {windows.length} 켜짐
          </span>
        </div>
        <Button onClick={() => setAdding(true)}>
          <PlusIcon className="size-3" />
          시간대 추가
        </Button>
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

      {adding && (
        <AddWindowSheet
          groups={groups}
          onAdd={handleAdd}
          onClose={() => setAdding(false)}
        />
      )}
    </>
  );
}
