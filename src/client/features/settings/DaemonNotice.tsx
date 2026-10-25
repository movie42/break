import { Button } from "@/shared/ui/Button";

interface DaemonNoticeProps {
  title: string;
  message: string;
  actionLabel: string;
  tone: "accent" | "warn";
  pending: boolean;
  onAction: () => void;
}

const TONE_CLASS = {
  accent: "border-accent-line bg-accent-soft",
  warn: "border-warn/25 bg-warn/8",
} as const;

export function DaemonNotice({
  title,
  message,
  actionLabel,
  tone,
  pending,
  onAction,
}: DaemonNoticeProps) {
  return (
    <section
      className={`mb-4 flex items-center justify-between gap-5 rounded-xl border px-4.5 py-3.5 ${TONE_CLASS[tone]}`}
    >
      <div className="flex flex-col gap-0.5">
        <span className="text-[13px] font-bold text-ink">{title}</span>
        <span className="text-[11.5px] text-ink-muted">{message}</span>
      </div>
      <Button disabled={pending} onClick={onAction} className="shrink-0 px-4.5">
        {actionLabel}
      </Button>
    </section>
  );
}
