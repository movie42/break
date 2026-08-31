interface NoticeProps {
  lines: string[];
}

export function Notice({ lines }: NoticeProps) {
  return (
    <div className="rounded-lg border border-notice-line bg-notice px-4 py-3 text-sm leading-relaxed text-notice-ink">
      {lines.map((line) => (
        <p key={line}>{line}</p>
      ))}
    </div>
  );
}
