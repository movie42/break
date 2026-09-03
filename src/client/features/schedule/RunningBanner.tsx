import { LockIcon } from "@/shared/ui/icons";

import type { BlockingSession } from "./active";
import { formatReleaseAt, formatRemaining } from "./active";

interface RunningBannerProps {
  session: BlockingSession;
  now: Date;
}

export function RunningBanner({ session, now }: RunningBannerProps) {
  const release =
    session.endsAt === null
      ? {
          headline: "풀리는 시각 없음",
          detail: "일주일 내내 막는 시간대가 켜져 있습니다",
        }
      : {
          headline: `${formatReleaseAt(now, session.endsAt)}에 풀립니다`,
          detail: `남은 시간 ${formatRemaining(now, session.endsAt)}`,
        };

  return (
    <section className="mb-4 flex items-center justify-between gap-5 rounded-xl border border-accent-line bg-accent-soft px-4.5 py-3.5">
      <div className="flex items-center gap-3">
        <span className="flex size-8 items-center justify-center rounded-full bg-accent text-accent-fg">
          <LockIcon className="size-4" />
        </span>
        <div className="flex flex-col gap-0.5">
          <span className="text-[13px] font-bold text-ink">
            지금 차단 중 — {release.headline}
          </span>
          <span className="text-[11.5px] text-ink-muted">
            {release.detail} · 실행 중인 시간대는 끄거나 지울 수 없습니다
          </span>
        </div>
      </div>
      <span className="shrink-0 text-[11px] font-semibold tracking-[0.12em] text-accent uppercase">
        {session.windowIds.length}개 실행
      </span>
    </section>
  );
}
