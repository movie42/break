import { useState } from "react";

import { Button } from "@/shared/ui/Button";
import { EmptyState } from "@/shared/ui/EmptyState";
import type { SiteTarget } from "@/shared/types";

interface SiteListProps {
  sites: SiteTarget[];
  onAdd: (input: string) => void;
  onRemove: (host: string) => void;
  error: string | null;
}

export function SiteList({ sites, onAdd, onRemove, error }: SiteListProps) {
  const [input, setInput] = useState("");

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onAdd(input);
    setInput("");
  }

  return (
    <section className="flex flex-col gap-4">
      <form onSubmit={handleSubmit} className="flex gap-2">
        <input
          value={input}
          onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
            setInput(event.target.value)
          }
          placeholder="youtube.com"
          aria-label="차단할 사이트 주소"
          className="flex-1 rounded-md border border-line bg-surface px-3 py-2 text-sm text-ink placeholder:text-ink-muted focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
        />
        <Button type="submit" disabled={input.trim().length === 0}>
          추가
        </Button>
      </form>

      {error && <p className="text-sm text-danger">{error}</p>}

      {sites.length === 0 ? (
        <EmptyState message="아직 추가한 사이트가 없습니다." />
      ) : (
        <ul className="divide-y divide-line overflow-hidden rounded-lg border border-line bg-surface">
          {sites.map((site) => (
            <li
              key={site.host}
              className="flex items-center justify-between px-4 py-3"
            >
              <span className="text-sm text-ink">{site.host}</span>
              <Button variant="danger" onClick={() => onRemove(site.host)}>
                삭제
              </Button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
