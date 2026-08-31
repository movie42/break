import type { AppState } from "@/shared/types";

interface StatusPanelProps {
  state: AppState;
}

function currentState(state: AppState): { label: string; detail: string } {
  const hasTargets = state.rules.sites.length > 0 || state.rules.apps.length > 0;
  const hasWindows = state.rules.schedule.windows.length > 0;

  if (!hasTargets || !hasWindows) {
    return {
      label: "규칙 없음",
      detail: "차단할 사이트와 시간대를 모두 정해야 판정이 시작됩니다.",
    };
  }
  if (state.blockingNow) {
    return {
      label: "차단 구간",
      detail: "지금 시각이 차단 구간 안에 있습니다.",
    };
  }
  return {
    label: "대기",
    detail: "지금 시각은 어느 차단 구간에도 속하지 않습니다.",
  };
}

export function StatusPanel({ state }: StatusPanelProps) {
  const { label, detail } = currentState(state);

  return (
    <section className="flex flex-col gap-4">
      <div className="rounded-lg border border-line bg-surface p-5">
        <p className="text-xs tracking-wide text-ink-muted">현재 상태</p>
        <p className="mt-1 text-2xl font-semibold text-ink">{label}</p>
        <p className="mt-2 text-sm text-ink-muted">{detail}</p>
      </div>

      <dl className="rounded-lg border border-line bg-surface p-4 text-sm">
        <div className="flex justify-between gap-4 py-1">
          <dt className="text-ink-muted">차단 사이트</dt>
          <dd className="text-ink">{state.rules.sites.length}개</dd>
        </div>
        <div className="flex justify-between gap-4 py-1">
          <dt className="text-ink-muted">차단 시간대</dt>
          <dd className="text-ink">{state.rules.schedule.windows.length}개</dd>
        </div>
        <div className="flex flex-col gap-1 py-1">
          <dt className="text-ink-muted">규칙 파일</dt>
          <dd className="break-all text-xs text-ink-muted">
            {state.rulesPath}
          </dd>
        </div>
      </dl>
    </section>
  );
}
