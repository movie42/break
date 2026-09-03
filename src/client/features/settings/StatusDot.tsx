import type { DaemonStatus } from "@/shared/types";

interface StatusDotProps {
  status: DaemonStatus;
  needsReinstall: boolean;
}

export function StatusDot({ status, needsReinstall }: StatusDotProps) {
  const running = status.kind === "running";
  const tone = !running ? "off" : needsReinstall ? "stale" : "on";

  const TONE_CLASS = {
    on: "bg-ok ring-ok/15",
    stale: "bg-warn ring-warn/15",
    off: "bg-danger ring-danger/15",
  } as const;

  const TONE_LABEL = {
    on: "실행 중",
    stale: "실행 중이지만 앱보다 오래됨",
    off: "중단됨",
  } as const;

  return (
    <span
      role="img"
      title={TONE_LABEL[tone]}
      aria-label={TONE_LABEL[tone]}
      className={`size-2.5 rounded-full ring-3 ${TONE_CLASS[tone]}`}
    />
  );
}
