import { useNow } from "@/shared/hooks/useNow";
import { LockIcon } from "@/shared/ui/icons";

import type { BlockingSession } from "./active";
import { formatCountdown, formatReleaseAt } from "./active";

const COUNTDOWN_MS = 1000;

interface RunningBannerProps {
  session: BlockingSession;
  onExtend: () => void;
}

export function RunningBanner({ session, onExtend }: RunningBannerProps) {
  const now = useNow(COUNTDOWN_MS);
  const running =
    session.windowIds.length + (session.instantEndsAt !== null ? 1 : 0);

  const release =
    session.endsAt === null
      ? {
          headline: "풀리는 시각 없음",
          detail: "일주일 내내 막는 시간대가 켜져 있습니다",
        }
      : {
          headline: `${formatReleaseAt(now, session.endsAt)}에 풀립니다`,
          detail: `남은 시간 ${formatCountdown(now, session.endsAt)}`,
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
          <span className="text-[11.5px] text-ink-muted tabular-nums">
            {release.detail} · 실행 중인 시간대는 끄거나 지울 수 없습니다
          </span>
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <span className="text-[11px] font-semibold tracking-[0.12em] text-accent uppercase">
          {running}개 실행
        </span>
        {session.endsAt !== null && (
          <button
            type="button"
            onClick={onExtend}
            className="h-7 rounded-lg border border-accent-line bg-surface px-3 text-xs font-semibold text-accent transition-colors hover:bg-accent-soft focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
          >
            연장
          </button>
        )}
      </div>
    </section>
  );
}
