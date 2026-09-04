import type { ButtonHTMLAttributes } from "react";

type Variant = "primary" | "secondary" | "quiet" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
}

const BASE =
  "inline-flex h-8 items-center gap-1.5 rounded-lg px-3.5 text-xs font-semibold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed";

const VARIANT_CLASS: Record<Variant, string> = {
  primary:
    "bg-accent text-accent-fg hover:bg-accent-hover disabled:bg-surface-muted disabled:text-ink-faint",
  secondary:
    "border border-line text-ink-soft hover:bg-surface-muted disabled:text-ink-faint",
  quiet: "font-medium text-ink-muted hover:text-ink disabled:text-ink-faint",
  danger:
    "border border-line font-semibold text-danger hover:bg-surface-muted disabled:text-ink-faint",
};

export function Button({
  variant = "primary",
  className,
  type = "button",
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={`${BASE} ${VARIANT_CLASS[variant]} ${className ?? ""}`}
      {...rest}
    />
  );
}
