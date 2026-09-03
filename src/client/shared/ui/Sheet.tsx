import { useEffect } from "react";
import type { ReactNode } from "react";

interface SheetProps {
  title: string;
  footerNote: string;
  confirmLabel: string;
  confirmDisabled: boolean;
  onConfirm: () => void;
  onClose: () => void;
  width: "narrow" | "wide";
  tone?: "accent" | "danger";
  children: ReactNode;
}

const CONFIRM_TONE: Record<"accent" | "danger", string> = {
  accent: "bg-accent hover:bg-accent-hover",
  danger: "bg-danger hover:bg-danger/85",
};

export function Sheet({
  title,
  footerNote,
  confirmLabel,
  confirmDisabled,
  onConfirm,
  onClose,
  width,
  tone = "accent",
  children,
}: SheetProps) {
  useEffect(() => {
    function handleKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
      }
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  return (
    <div className="absolute inset-0 z-20 flex items-center justify-center bg-ink/20 p-6">
      <div
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className={`rounded-2xl border border-accent-line bg-surface px-6 pt-6 pb-5 shadow-sheet ${
          width === "wide" ? "w-[528px]" : "w-[420px]"
        }`}
      >
        <p className="mb-5 text-[17px] font-bold text-ink">{title}</p>

        {children}

        <div className="flex items-center justify-between border-t border-surface-muted pt-4">
          <span className="text-[11.5px] text-ink-faint">{footerNote}</span>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              className="h-8 rounded-lg px-4 text-xs font-medium text-ink-muted transition-colors hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              취소
            </button>
            <button
              type="button"
              onClick={onConfirm}
              disabled={confirmDisabled}
              className={`h-8 rounded-lg px-5 text-xs font-semibold text-accent-fg transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:bg-surface-muted disabled:text-ink-faint ${CONFIRM_TONE[tone]}`}
            >
              {confirmLabel}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
