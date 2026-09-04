import { useState } from "react";

import type { SiteGroup, TimeWindow } from "@/shared/types";
import { Button } from "@/shared/ui/Button";
import { EmptyState } from "@/shared/ui/EmptyState";
import { PlusIcon } from "@/shared/ui/icons";

import { CreateGroupSheet } from "./CreateGroupSheet";
import { GroupCard } from "./GroupCard";

interface GroupsTabProps {
  groups: SiteGroup[];
  windows: TimeWindow[];
  lockedGroupIds: string[];
  onCreateGroup: (name: string) => void;
  onRenameGroup: (groupId: string, name: string) => void;
  onRemoveGroup: (groupId: string) => void;
  onAddSite: (groupId: string, input: string) => void;
  onRemoveSite: (groupId: string, host: string) => void;
}

export function GroupsTab({
  groups,
  windows,
  lockedGroupIds,
  onCreateGroup,
  onRenameGroup,
  onRemoveGroup,
  onAddSite,
  onRemoveSite,
}: GroupsTabProps) {
  const [creating, setCreating] = useState(false);

  function handleCreate(name: string) {
    onCreateGroup(name);
    setCreating(false);
  }

  return (
    <>
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-baseline gap-2.5">
          <span className="text-[13px] font-bold text-ink">그룹</span>
          <span className="text-xs text-ink-faint">{groups.length}</span>
        </div>
        <Button onClick={() => setCreating(true)}>
          <PlusIcon className="size-3" />
          그룹 만들기
        </Button>
      </div>

      {groups.length === 0 ? (
        <EmptyState
          title="아직 그룹이 없습니다"
          hint="막고 싶은 사이트를 그룹으로 묶어 시간대에 붙입니다"
        />
      ) : (
        <ul className="flex list-none flex-col gap-2.5 p-0">
          {groups.map((group) => (
            <GroupCard
              key={group.id}
              group={group}
              usedInWindows={
                windows.filter((window) => window.groupIds.includes(group.id))
                  .length
              }
              locked={lockedGroupIds.includes(group.id)}
              onRename={onRenameGroup}
              onAddSite={onAddSite}
              onRemoveSite={onRemoveSite}
              onRemoveGroup={onRemoveGroup}
            />
          ))}
        </ul>
      )}

      <p className="mt-3.5 px-0.5 text-[11.5px] leading-relaxed text-ink-faint">
        그룹은 시간대에 붙여서 씁니다. 밤에는 소셜만, 낮에는 소셜과 쇼핑을 함께
        막는 식으로 시간대마다 다른 그룹을 고를 수 있습니다. 실행 중인 시간대가
        쓰는 그룹은 주소를 넣을 수만 있고, 빼거나 지울 수 없습니다.
      </p>

      {creating && (
        <CreateGroupSheet
          onCreate={handleCreate}
          onClose={() => setCreating(false)}
        />
      )}
    </>
  );
}
