import { Sheet } from "./Sheet";

interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel: string;
  tone?: "accent" | "danger";
  onConfirm: () => void;
  onClose: () => void;
}

export function ConfirmDialog({
  title,
  message,
  confirmLabel,
  tone = "danger",
  onConfirm,
  onClose,
}: ConfirmDialogProps) {
  return (
    <Sheet
      title={title}
      width="narrow"
      tone={tone}
      footerNote=""
      confirmLabel={confirmLabel}
      confirmDisabled={false}
      onConfirm={onConfirm}
      onClose={onClose}
    >
      <p className="mb-5 text-[13px] leading-relaxed text-ink-muted">
        {message}
      </p>
    </Sheet>
  );
}
