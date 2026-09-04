interface EmptyStateProps {
  title: string;
  hint?: string;
}

export function EmptyState({ title, hint }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center gap-1.5 rounded-xl border border-dashed border-line-strong px-5 py-10">
      <p className="text-[13px] text-ink-muted">{title}</p>
      {hint !== undefined && (
        <p className="text-[11.5px] text-ink-faint">{hint}</p>
      )}
    </div>
  );
}
