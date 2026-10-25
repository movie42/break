import { useState } from "react";

import type { DaemonStatus, DnsGuardStatus } from "@/shared/types";
import { Button } from "@/shared/ui/Button";
import { ConfirmDialog } from "@/shared/ui/ConfirmDialog";

const UNINSTALL_MESSAGE =
  "제거하면 차단이 즉시 풀리고, 다시 쓰려면 관리자 암호를 다시 입력해야 합니다.";

const STATUS_LABEL: Record<DaemonStatus["kind"], string> = {
  notInstalled: "설치 안 됨",
  installed: "정지",
  running: "실행 중",
};

const DNS_GUARD_LABEL: Record<DnsGuardStatus["kind"], string | null> = {
  off: null,
  applied: "보안 DNS 차단: 적용 중",
  failed: "보안 DNS 차단: 실패",
};

interface SettingsPopoverProps {
  status: DaemonStatus;
  dnsGuard: DnsGuardStatus;
  needsReinstall: boolean;
  blocking: boolean;
  rulesPath: string;
  pending: boolean;
  error: string | null;
  onInstall: () => void;
  onUninstall: () => void;
  onClose: () => void;
}

export function SettingsPopover({
  status,
  dnsGuard,
  needsReinstall,
  blocking,
  rulesPath,
  pending,
  error,
  onInstall,
  onUninstall,
  onClose,
}: SettingsPopoverProps) {
  const [confirming, setConfirming] = useState(false);
  const installed = status.kind !== "notInstalled";
  const running = status.kind === "running";
  const tone = !running
    ? "text-danger"
    : needsReinstall
      ? "text-warn"
      : "text-ok";
  const dot = !running ? "bg-danger" : needsReinstall ? "bg-warn" : "bg-ok";
  const dnsGuardLabel = DNS_GUARD_LABEL[dnsGuard.kind];

  function handleUninstall() {
    setConfirming(false);
    onUninstall();
  }

  return (
    <>
      <button
        type="button"
        aria-label="설정 닫기"
        onClick={onClose}
        className="absolute inset-0 z-10 cursor-default bg-ink/10"
      />
      <div className="absolute top-13 right-5.5 z-10 w-74 rounded-xl border border-accent-line bg-surface px-4 pt-4 pb-3.5 shadow-pop">
        <div className="mb-3 flex items-center justify-between">
          <span className="text-[12.5px] font-bold text-ink">
            차단 프로그램
          </span>
          <span
            className={`inline-flex items-center gap-1.5 text-[11.5px] ${tone}`}
          >
            <span className={`size-1.5 rounded-full ${dot}`} />
            {STATUS_LABEL[status.kind]}
          </span>
        </div>

        {installed ? (
          <div className="flex gap-2">
            <Button
              disabled={pending}
              onClick={onInstall}
              className="flex-1 justify-center"
            >
              다시 설치
            </Button>
            <Button
              variant="danger"
              disabled={pending || blocking}
              title={blocking ? "차단 중에는 제거할 수 없습니다" : undefined}
              onClick={() => setConfirming(true)}
              className="flex-1 justify-center"
            >
              제거
            </Button>
          </div>
        ) : (
          <Button
            disabled={pending}
            onClick={onInstall}
            className="w-full justify-center"
          >
            설치
          </Button>
        )}

        {blocking && (
          <p className="mt-2.5 text-[11px] leading-relaxed text-accent">
            지금 차단 중입니다. 끝날 때까지 제거할 수 없습니다.
          </p>
        )}

        {installed && (
          <p
            className={`mt-2.5 text-[11px] leading-relaxed ${
              needsReinstall ? "text-warn" : "text-ink-faint"
            }`}
          >
            {needsReinstall
              ? "설치된 것이 앱과 다릅니다. 다시 설치해야 바뀐 규칙을 읽습니다."
              : "설치된 것이 앱과 같습니다."}
          </p>
        )}

        {dnsGuardLabel !== null && (
          <p
            className={`mt-2.5 text-[11px] leading-relaxed ${
              dnsGuard.kind === "failed" ? "text-warn" : "text-ink-faint"
            }`}
          >
            {dnsGuardLabel}
          </p>
        )}

        {error !== null && (
          <p className="mt-2.5 text-[11.5px] leading-relaxed text-danger">
            {error}
          </p>
        )}

        <div className="mt-3 border-t border-surface-muted pt-3">
          <span className="mb-1.5 block text-[10.5px] font-bold tracking-[0.1em] uppercase text-ink-faint">
            규칙 파일
          </span>
          <span className="block text-[11px] leading-normal break-all text-ink-faint">
            {rulesPath}
          </span>
        </div>
      </div>

      {confirming && (
        <ConfirmDialog
          title="차단 프로그램 제거"
          message={UNINSTALL_MESSAGE}
          confirmLabel="제거"
          onConfirm={handleUninstall}
          onClose={() => setConfirming(false)}
        />
      )}
    </>
  );
}
