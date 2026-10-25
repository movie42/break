import { EmptyState } from "@/shared/ui/EmptyState";

import { DowntimeBanner } from "./DowntimeBanner";
import { describeEvent, formatStamp } from "./describe";
import { useRecentEvents } from "./useRecentEvents";

export function HistoryTab() {
  const events = useRecentEvents();

  return (
    <>
      <DowntimeBanner />
      {events.length === 0 ? (
        <EmptyState
          title="아직 남은 기록이 없습니다"
          hint="차단 프로그램을 설치하면 켜고 끈 기록이 여기에 쌓입니다"
        />
      ) : (
        <ul className="flex list-none flex-col p-0">
          {events.map((event, index) => (
            <li
              key={`${event.at}-${event.kind}-${index}`}
              className="flex gap-4 border-b border-surface-muted py-2 text-[12.5px]"
            >
              <span className="w-24 shrink-0 text-ink-faint tabular-nums">
                {formatStamp(event.at)}
              </span>
              <span
                className={
                  event.kind === "down" || event.kind === "mailFailed"
                    ? "font-semibold text-danger"
                    : "text-ink"
                }
              >
                {describeEvent(event)}
              </span>
            </li>
          ))}
        </ul>
      )}
      <p className="mt-3.5 px-0.5 text-[11.5px] leading-relaxed text-ink-faint">
        기록은 관리자만 지울 수 있는 자리(/Library/Logs/Break)에 남습니다. 차단
        프로그램이 몇 초마다 살아 있다는 표시를 남기고, 다시 켜질 때 그 사이의
        빈 구간을 꺼져 있던 시간으로 적습니다.
      </p>
    </>
  );
}
