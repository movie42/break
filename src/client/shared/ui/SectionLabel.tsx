import type { ReactNode } from "react";

interface SectionLabelProps {
  children: ReactNode;
  tone?: "muted" | "accent";
}

export function SectionLabel({ children, tone = "muted" }: SectionLabelProps) {
  return (
    <span
      className={`text-[11px] font-bold tracking-[0.12em] uppercase ${
        tone === "accent" ? "text-accent" : "text-ink-muted"
      }`}
    >
      {children}
    </span>
  );
}
