import { useState } from "react";

import { SectionLabel } from "@/shared/ui/SectionLabel";
import { Sheet } from "@/shared/ui/Sheet";

import type { GroupPreset } from "./presets";
import { GROUP_PRESETS } from "./presets";

interface CreateGroupSheetProps {
  usedPresets: string[];
  onCreate: (name: string) => void;
  onCreatePreset: (preset: GroupPreset) => void;
  onClose: () => void;
}

export function CreateGroupSheet({
  usedPresets,
  onCreate,
  onCreatePreset,
  onClose,
}: CreateGroupSheetProps) {
  const [name, setName] = useState("");
  const [previewId, setPreviewId] = useState<string | null>(null);
  const trimmed = name.trim();
  const preview = GROUP_PRESETS.find((preset) => preset.id === previewId);

  return (
    <Sheet
      title="그룹 만들기"
      width="wide"
      footerNote="만든 뒤 주소를 넣거나 뺄 수 있습니다"
      confirmLabel="만들기"
      confirmDisabled={trimmed.length === 0}
      onConfirm={() => onCreate(trimmed)}
      onClose={onClose}
    >
      <div className="mb-2">
        <SectionLabel>추천에서 고르기</SectionLabel>
      </div>
      <div className="mb-2.5 flex flex-wrap gap-1.5">
        {GROUP_PRESETS.map((preset) => {
          const used = usedPresets.includes(preset.id);
          return (
            <button
              key={preset.id}
              type="button"
              disabled={used}
              onMouseEnter={() => setPreviewId(preset.id)}
              onFocus={() => setPreviewId(preset.id)}
              onClick={() => onCreatePreset(preset)}
              className="inline-flex items-center rounded-lg border border-line-strong bg-surface px-3.5 py-1.5 text-[12.5px] font-medium text-ink-muted transition-colors hover:border-accent hover:text-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:border-line disabled:text-ink-ghost"
            >
              {preset.name}
              {used && <span className="ml-1.5 text-[11px]">만듦</span>}
            </button>
          );
        })}
      </div>
      <p className="mb-5 min-h-9 text-[11.5px] leading-relaxed text-ink-faint">
        {preview === undefined
          ? "누르면 주소가 들어 있는 그룹이 바로 생깁니다."
          : `${preview.name} — ${preview.hosts.length}개: ${preview.hosts.join(", ")}`}
      </p>

      <div className="mb-2">
        <SectionLabel>또는 직접 만들기</SectionLabel>
      </div>
      <input
        value={name}
        aria-label="그룹 이름"
        placeholder="예: 소셜"
        onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
          setName(event.target.value)
        }
        className="mb-5 h-10 w-full rounded-lg border border-line-strong bg-surface px-3.5 text-sm text-ink outline-none placeholder:text-ink-faint focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
      />
    </Sheet>
  );
}
