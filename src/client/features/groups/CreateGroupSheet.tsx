import { useState } from "react";

import { SectionLabel } from "@/shared/ui/SectionLabel";
import { Sheet } from "@/shared/ui/Sheet";

interface CreateGroupSheetProps {
  onCreate: (name: string) => void;
  onClose: () => void;
}

export function CreateGroupSheet({ onCreate, onClose }: CreateGroupSheetProps) {
  const [name, setName] = useState("");
  const trimmed = name.trim();

  return (
    <Sheet
      title="그룹 만들기"
      width="narrow"
      footerNote="만든 뒤 주소를 넣습니다"
      confirmLabel="만들기"
      confirmDisabled={trimmed.length === 0}
      onConfirm={() => onCreate(trimmed)}
      onClose={onClose}
    >
      <div className="mb-2">
        <SectionLabel>이름</SectionLabel>
      </div>
      <input
        value={name}
        aria-label="그룹 이름"
        placeholder="예: 소셜"
        autoFocus
        onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
          setName(event.target.value)
        }
        className="mb-5 h-10 w-full rounded-lg border border-line-strong bg-surface px-3.5 text-sm text-ink outline-none placeholder:text-ink-faint focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent"
      />
    </Sheet>
  );
}
