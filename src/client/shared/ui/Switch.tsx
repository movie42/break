interface SwitchProps {
  checked: boolean;
  label: string;
  disabled?: boolean;
  onChange: (next: boolean) => void;
}

export function Switch({
  checked,
  label,
  disabled = false,
  onChange,
}: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-5.5 w-9.5 shrink-0 items-center rounded-full border transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed ${
        checked
          ? "border-accent bg-accent"
          : "border-line-strong bg-surface-muted"
      }`}
    >
      <span
        className={`size-4 rounded-full bg-surface shadow-card transition-transform ${
          checked ? "translate-x-4.5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}
