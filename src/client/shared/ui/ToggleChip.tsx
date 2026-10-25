import type { ReactNode } from "react";

interface ToggleChipProps {
  selected: boolean;
  shape: "circle" | "pill";
  onClick: () => void;
  children: ReactNode;
}

const SHAPE_CLASS: Record<ToggleChipProps["shape"], string> = {
  circle: "size-9 justify-center rounded-full text-[12.5px]",
  pill: "rounded-lg px-3.5 py-1.5 text-[12.5px]",
};

export function ToggleChip({
  selected,
  shape,
  onClick,
  children,
}: ToggleChipProps) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onClick}
      className={`inline-flex items-center border font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
        SHAPE_CLASS[shape]
      } ${
        selected
          ? "border-accent bg-accent-soft text-accent"
          : "border-line-strong bg-surface text-ink-muted hover:bg-surface-muted"
      }`}
    >
      {children}
    </button>
  );
}
