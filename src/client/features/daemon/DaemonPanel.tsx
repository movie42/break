import type { DaemonStatus } from "@/shared/types";
import { Button } from "@/shared/ui/Button";

const UNINSTALL_CONFIRM = "제거하면 차단이 즉시 풀립니다. 계속할까요?";

const STATUS_TEXT: Record<DaemonStatus["kind"], { label: string; detail: string }> = {
  notInstalled: {
    label: "미설치",
    detail: "차단 프로그램이 설치되어 있지 않아 지금은 아무것도 차단하지 않습니다.",
  },
  installed: {
    label: "설치됨 · 정지",
    detail: "설치는 되어 있지만 프로그램이 돌고 있지 않습니다.",
  },
  running: {
    label: "실행 중",
    detail: "앱을 닫아도 시간대에 맞춰 차단합니다.",
  },
};

interface DaemonPanelProps {
  status: DaemonStatus;
  pending: boolean;
  error: string | null;
  onInstall: () => void;
  onUninstall: () => void;
}

export function DaemonPanel({
  status,
  pending,
  error,
  onInstall,
  onUninstall,
}: DaemonPanelProps) {
  const { label, detail } = STATUS_TEXT[status.kind];

  function handleUninstall() {
    if (window.confirm(UNINSTALL_CONFIRM)) {
      onUninstall();
    }
  }

  return (
    <section className="flex flex-col gap-3 rounded-lg border border-line bg-surface p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs tracking-wide text-ink-muted">차단 프로그램</p>
          <p className="mt-1 text-lg font-semibold text-ink">{label}</p>
          <p className="mt-1 text-sm text-ink-muted">{detail}</p>
        </div>
        {status.kind === "notInstalled" ? (
          <Button onClick={onInstall} disabled={pending}>
            차단 프로그램 설치
          </Button>
        ) : (
          <Button variant="danger" onClick={handleUninstall} disabled={pending}>
            차단 프로그램 제거
          </Button>
        )}
      </div>

      {error !== null && <p className="text-sm text-danger">{error}</p>}
    </section>
  );
}
