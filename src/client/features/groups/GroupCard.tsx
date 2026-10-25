import { useState } from "react";

import type { SiteGroup } from "@/shared/types";
import { CheckIcon, CloseIcon, LockIcon, PencilIcon } from "@/shared/ui/icons";
import { IconButton } from "@/shared/ui/IconButton";

interface GroupCardProps {
  group: SiteGroup;
  usedInWindows: number;
  locked: boolean;
  onRename: (groupId: string, name: string) => void;
  onAddSite: (groupId: string, input: string) => void;
  onRemoveSite: (groupId: string, host: string) => void;
  onRemoveGroup: (groupId: string) => void;
}

export function GroupCard({
  group,
  usedInWindows,
  locked,
  onRename,
  onAddSite,
  onRemoveSite,
  onRemoveGroup,
}: GroupCardProps) {
  const [draft, setDraft] = useState("");
  const [renaming, setRenaming] = useState(false);
  const [name, setName] = useState(group.name);

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (draft.trim().length === 0) {
      return;
    }
    onAddSite(group.id, draft);
    setDraft("");
  }

  function startRename() {
    setName(group.name);
    setRenaming(true);
  }

  function commitRename(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = name.trim();
    setRenaming(false);
    if (trimmed.length > 0 && trimmed !== group.name) {
      onRename(group.id, trimmed);
    }
  }

  function handleRenameKey(event: React.KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Escape") {
      setRenaming(false);
    }
  }

  return (
    <li
      className={`rounded-xl border bg-surface px-4 pt-4 pb-3.5 shadow-card ${
        locked ? "border-accent-line" : "border-line"
      }`}
    >
      <div className="mb-3 flex items-center justify-between gap-3">
        {renaming ? (
          <form onSubmit={commitRename} className="flex items-center gap-1.5">
            <input
              value={name}
              aria-label="그룹 이름"
              autoFocus
              onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                setName(event.target.value)
              }
              onKeyDown={handleRenameKey}
              className="h-7 w-44 rounded-lg border border-line-strong bg-surface px-2.5 text-sm font-bold text-ink outline-none focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
            />
            <IconButton label="이름 저장" type="submit">
              <CheckIcon className="size-3.5" />
            </IconButton>
            <IconButton
              label="이름 수정 취소"
              onClick={() => setRenaming(false)}
            >
              <CloseIcon className="size-3.5" />
            </IconButton>
          </form>
        ) : (
          <div className="group/name flex min-w-0 items-baseline gap-2.5">
            <button
              type="button"
              onClick={startRename}
              title="이름 수정"
              className="truncate text-sm font-bold text-ink transition-colors hover:text-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              {group.name}
            </button>
            <IconButton
              label="이름 수정"
              onClick={startRename}
              className="size-5 opacity-0 group-hover/name:opacity-100 focus-visible:opacity-100"
            >
              <PencilIcon className="size-3" />
            </IconButton>
            <span className="shrink-0 text-[11px] text-ink-faint">
              {group.sites.length}개 주소 · 시간대 {usedInWindows}곳에서 사용
            </span>
          </div>
        )}

        {locked ? (
          <span
            title="이 그룹을 쓰는 시간대가 실행 중입니다"
            className="inline-flex shrink-0 items-center gap-1.5 rounded-full border border-accent-line bg-accent-soft px-2.5 py-0.5 text-[11px] font-semibold text-accent"
          >
            <LockIcon className="size-3" />
            실행 중
          </span>
        ) : (
          <IconButton label="그룹 삭제" onClick={() => onRemoveGroup(group.id)}>
            <CloseIcon className="size-3.5" />
          </IconButton>
        )}
      </div>

      <div className="mb-3 flex flex-wrap gap-1.5">
        {group.sites.map((site) => (
          <span
            key={site.host}
            className="inline-flex items-center gap-1.5 rounded-lg border border-line bg-bg py-1 pr-2 pl-2.5 text-xs text-ink-soft transition-colors hover:bg-surface-muted"
          >
            {site.host}
            {!locked && (
              <button
                type="button"
                aria-label={`${site.host} 빼기`}
                onClick={() => onRemoveSite(group.id, site.host)}
                className="flex text-ink-faint transition-colors hover:text-danger focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
              >
                <CloseIcon className="size-2.5" />
              </button>
            )}
          </span>
        ))}
        <form onSubmit={handleSubmit} className="inline-flex">
          <input
            value={draft}
            aria-label={`${group.name} 그룹에 주소 추가`}
            placeholder="주소 추가"
            onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
              setDraft(event.target.value)
            }
            className="h-7 w-36 rounded-lg border border-dashed border-line-strong bg-transparent px-2.5 text-xs text-ink outline-none placeholder:text-ink-faint focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
          />
        </form>
      </div>

      <div className="flex items-center gap-2.5 border-t border-surface-muted pt-3">
        <span className="text-[11px] font-bold tracking-[0.1em] uppercase text-ink-faint">
          앱
        </span>
        <span className="rounded-full border border-line px-2 py-0.5 text-[10px] text-ink-faint">
          준비 중
        </span>
        {group.apps.map((app) => (
          <span
            key={app.bundleId}
            className="rounded-lg border border-surface-muted px-2.5 py-0.5 text-[11.5px] text-ink-faint"
          >
            {app.displayName}
          </span>
        ))}
      </div>
    </li>
  );
}
