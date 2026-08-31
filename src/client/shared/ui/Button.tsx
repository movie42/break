import type { ButtonHTMLAttributes } from "react";

type Variant = "primary" | "ghost" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
}

const VARIANT_CLASS: Record<Variant, string> = {
  primary:
    "bg-accent text-accent-fg hover:bg-accent-hover disabled:opacity-40 disabled:hover:bg-accent",
  ghost:
    "border border-line bg-surface text-ink hover:bg-surface-muted disabled:opacity-40",
  danger: "text-danger hover:bg-surface-muted disabled:opacity-40",
};

export function Button({ variant = "primary", className, ...rest }: ButtonProps) {
  return (
    <button
      className={`rounded-md px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${VARIANT_CLASS[variant]} ${className ?? ""}`}
      {...rest}
    />
  );
}
