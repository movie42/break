interface EmptyStateProps {
  message: string;
}

export function EmptyState({ message }: EmptyStateProps) {
  return (
    <p className="rounded-lg border border-dashed border-line px-4 py-8 text-center text-sm text-ink-muted">
      {message}
    </p>
  );
}
