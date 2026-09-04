import { useEffect, useState } from "react";

import { GroupsTab } from "@/features/groups/GroupsTab";
import { coversNow, currentSession } from "@/features/schedule/active";
import { ScheduleTab } from "@/features/schedule/ScheduleTab";
import { DaemonNotice } from "@/features/settings/DaemonNotice";
import { SettingsPopover } from "@/features/settings/SettingsPopover";
import { StatusDot } from "@/features/settings/StatusDot";
import {
  installDaemon,
  loadRules,
  quitBrowsers,
  runningBrowsers,
  saveRules,
  uninstallDaemon,
} from "@/shared/api/rules";
import { useNow } from "@/shared/hooks/useNow";
import type { AppState, Rules, TimeWindow } from "@/shared/types";
import { ConfirmDialog } from "@/shared/ui/ConfirmDialog";
import { GearIcon } from "@/shared/ui/icons";
import { IconButton } from "@/shared/ui/IconButton";

const TABS = [
  { id: "schedule", label: "시간대" },
  { id: "sites", label: "사이트" },
] as const;

const TICK_MS = 5000;

type TabId = (typeof TABS)[number]["id"];

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [tab, setTab] = useState<TabId>("schedule");
  const [error, setError] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [daemonPending, setDaemonPending] = useState(false);
  const [daemonError, setDaemonError] = useState<string | null>(null);
  const [openBrowsers, setOpenBrowsers] = useState<string[] | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const now = useNow(TICK_MS);

  useEffect(() => {
    loadRules()
      .then(setState)
      .catch((cause: unknown) => setError(String(cause)));
  }, []);

  const windows = state?.rules.schedule.windows ?? [];
  const session = currentSession(windows, now);
  const runningWindowIds = session?.windowIds ?? [];
  const lockedGroupIds = session?.groupIds ?? [];

  async function commit(rules: Rules) {
    try {
      const next = await saveRules(rules);
      setState(next);
      setError(
        next.rejectedSites.length > 0
          ? `주소로 읽을 수 없어 저장하지 않았습니다: ${next.rejectedSites.join(", ")}`
          : null,
      );
    } catch (cause: unknown) {
      setError(String(cause));
    }
  }

  async function runDaemonAction(action: () => Promise<AppState>) {
    setDaemonPending(true);
    setDaemonError(null);
    try {
      setState(await action());
      setSettingsOpen(false);
    } catch (cause: unknown) {
      setDaemonError(String(cause));
    } finally {
      setDaemonPending(false);
    }
  }

  async function askToQuitBrowsers() {
    const open = await runningBrowsers().catch(() => []);
    if (open.length > 0) {
      setOpenBrowsers(open);
    }
  }

  function handleAddWindow(window: TimeWindow) {
    if (!state) return;
    void commit({
      ...state.rules,
      schedule: {
        ...state.rules.schedule,
        windows: [...state.rules.schedule.windows, window],
      },
    });
    setNotice("시간대를 만들었습니다. 목록에서 켜야 실행됩니다.");
  }

  async function handleToggleWindow(id: string, next: boolean) {
    if (!state) return;

    const target = state.rules.schedule.windows.find(
      (window) => window.id === id,
    );
    if (target === undefined) return;

    if (!next && runningWindowIds.includes(id)) {
      setError(
        "실행 중인 시간대는 끌 수 없습니다. 끝날 때까지 기다려야 합니다.",
      );
      return;
    }

    setError(null);
    setNotice(null);
    await commit({
      ...state.rules,
      schedule: {
        ...state.rules.schedule,
        windows: state.rules.schedule.windows.map((window) =>
          window.id === id ? { ...window, enabled: next } : window,
        ),
      },
    });

    if (next && target.groupIds.length > 0 && coversNow(target, new Date())) {
      await askToQuitBrowsers();
    }
  }

  async function handleQuitBrowsers() {
    setOpenBrowsers(null);
    const report = await quitBrowsers().catch(() => null);
    if (report === null) {
      setNotice("브라우저를 닫지 못했습니다.");
      return;
    }
    setNotice(
      report.stillOpen.length > 0
        ? `${report.stillOpen.join(", ")}은 닫히지 않았습니다. 직접 종료해 주세요.`
        : `${report.closed.join(", ")}을 닫았습니다.`,
    );
  }

  function handleRemoveWindow(id: string) {
    if (!state) return;
    if (runningWindowIds.includes(id)) {
      setError("실행 중인 시간대는 지울 수 없습니다.");
      return;
    }
    void commit({
      ...state.rules,
      schedule: {
        ...state.rules.schedule,
        windows: state.rules.schedule.windows.filter(
          (window) => window.id !== id,
        ),
      },
    });
  }

  function handleCreateGroup(name: string) {
    if (!state) return;
    void commit({
      ...state.rules,
      groups: [
        ...state.rules.groups,
        { id: crypto.randomUUID(), name, sites: [], apps: [] },
      ],
    });
  }

  function handleRenameGroup(groupId: string, name: string) {
    if (!state) return;
    void commit({
      ...state.rules,
      groups: state.rules.groups.map((group) =>
        group.id === groupId ? { ...group, name } : group,
      ),
    });
  }

  function handleRemoveGroup(groupId: string) {
    if (!state) return;
    if (lockedGroupIds.includes(groupId)) {
      setError("실행 중인 시간대가 쓰는 그룹입니다. 지울 수 없습니다.");
      return;
    }
    void commit({
      ...state.rules,
      groups: state.rules.groups.filter((group) => group.id !== groupId),
      schedule: {
        ...state.rules.schedule,
        windows: state.rules.schedule.windows.map((window) => ({
          ...window,
          groupIds: window.groupIds.filter((id) => id !== groupId),
        })),
      },
    });
  }

  function handleAddSite(groupId: string, input: string) {
    if (!state) return;
    void commit({
      ...state.rules,
      groups: state.rules.groups.map((group) =>
        group.id === groupId
          ? { ...group, sites: [...group.sites, { host: input.trim() }] }
          : group,
      ),
    });
  }

  function handleRemoveSite(groupId: string, host: string) {
    if (!state) return;
    if (lockedGroupIds.includes(groupId)) {
      setError("실행 중인 시간대가 쓰는 그룹입니다. 주소를 뺄 수 없습니다.");
      return;
    }
    void commit({
      ...state.rules,
      groups: state.rules.groups.map((group) =>
        group.id === groupId
          ? {
              ...group,
              sites: group.sites.filter((site) => site.host !== host),
            }
          : group,
      ),
    });
  }

  return (
    <main className="relative flex h-full flex-col overflow-hidden bg-bg">
      <header className="flex h-14.5 shrink-0 items-center justify-between border-b border-line bg-surface px-6.5">
        <span className="text-xl font-black tracking-[0.02em] text-ink">
          BREAK
        </span>
        <div className="flex items-center gap-4.5">
          <IconButton label="설정" onClick={() => setSettingsOpen((on) => !on)}>
            <GearIcon className="size-[17px]" />
          </IconButton>
          {state !== null && (
            <StatusDot
              status={state.daemon}
              needsReinstall={state.daemonNeedsReinstall}
            />
          )}
        </div>
      </header>

      <nav className="flex h-11 shrink-0 items-stretch gap-6.5 border-b border-line bg-surface px-6.5">
        {TABS.map((item) => (
          <button
            key={item.id}
            type="button"
            aria-current={tab === item.id ? "page" : undefined}
            onClick={() => setTab(item.id)}
            className={`-mb-px flex items-center border-b-2 text-[13px] font-semibold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
              tab === item.id
                ? "border-accent text-ink"
                : "border-transparent text-ink-faint hover:text-ink-muted"
            }`}
          >
            {item.label}
          </button>
        ))}
      </nav>

      <div className="min-h-0 flex-1 overflow-y-auto px-6.5 pt-4.5 pb-6">
        {state === null ? (
          <p className="text-[13px] text-ink-muted">규칙을 읽는 중입니다.</p>
        ) : (
          <>
            {state.daemon.kind === "notInstalled" && (
              <DaemonNotice
                tone="accent"
                title="차단 프로그램을 설치해야 시작합니다"
                message="설치할 때 Mac 관리자 암호를 한 번 물어봅니다."
                actionLabel="설치"
                pending={daemonPending}
                onAction={() => void runDaemonAction(installDaemon)}
              />
            )}

            {state.daemonNeedsReinstall && (
              <DaemonNotice
                tone="warn"
                title="차단 프로그램이 앱보다 오래됐습니다"
                message="지금 규칙을 제대로 못 읽고 있을 수 있습니다. 다시 설치하면 맞춰집니다."
                actionLabel="다시 설치"
                pending={daemonPending}
                onAction={() => void runDaemonAction(installDaemon)}
              />
            )}

            {error !== null && (
              <p className="mb-3 text-[13px] text-danger">{error}</p>
            )}

            {notice !== null && (
              <p className="mb-3 text-[13px] text-ink-muted">{notice}</p>
            )}

            {tab === "schedule" && (
              <ScheduleTab
                windows={windows}
                groups={state.rules.groups}
                session={session}
                now={now}
                onAdd={handleAddWindow}
                onToggle={(id, next) => void handleToggleWindow(id, next)}
                onRemove={handleRemoveWindow}
              />
            )}

            {tab === "sites" && (
              <GroupsTab
                groups={state.rules.groups}
                windows={windows}
                lockedGroupIds={lockedGroupIds}
                onCreateGroup={handleCreateGroup}
                onRenameGroup={handleRenameGroup}
                onRemoveGroup={handleRemoveGroup}
                onAddSite={handleAddSite}
                onRemoveSite={handleRemoveSite}
              />
            )}
          </>
        )}
      </div>

      {openBrowsers !== null && (
        <ConfirmDialog
          tone="accent"
          title="지금 바로 적용할까요?"
          message={`방금 켠 시간대는 지금 차단 구간입니다. ${openBrowsers.join(", ")}이(가) 열려 있는데, 브라우저는 자기 캐시를 들고 있어서 껐다 켜야 바로 막힙니다. 탭은 다시 켤 때 복원됩니다.`}
          confirmLabel="브라우저 닫기"
          onConfirm={() => void handleQuitBrowsers()}
          onClose={() => setOpenBrowsers(null)}
        />
      )}

      {settingsOpen && state !== null && (
        <SettingsPopover
          status={state.daemon}
          dnsGuard={state.dnsGuard}
          needsReinstall={state.daemonNeedsReinstall}
          blocking={session !== null}
          rulesPath={state.rulesPath}
          pending={daemonPending}
          error={daemonError}
          onInstall={() => void runDaemonAction(installDaemon)}
          onUninstall={() => void runDaemonAction(uninstallDaemon)}
          onClose={() => setSettingsOpen(false)}
        />
      )}
    </main>
  );
}
