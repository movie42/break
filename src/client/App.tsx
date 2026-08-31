import { useEffect, useState } from "react";

import { ScheduleEditor } from "@/features/schedule/ScheduleEditor";
import { SiteList } from "@/features/sites/SiteList";
import { StatusPanel } from "@/features/status/StatusPanel";
import { loadRules, saveRules } from "@/shared/api/rules";
import type { AppState, Rules, TimeWindow } from "@/shared/types";
import { Notice } from "@/shared/ui/Notice";

const NOTICE_LINES = [
  "아직 차단은 동작하지 않습니다.",
  "지금은 사이트와 시간대를 저장하는 것까지만 가능합니다.",
];

const TABS = [
  { id: "status", label: "상태" },
  { id: "sites", label: "사이트" },
  { id: "schedule", label: "시간대" },
] as const;

type TabId = (typeof TABS)[number]["id"];

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [tab, setTab] = useState<TabId>("status");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadRules()
      .then(setState)
      .catch((cause: unknown) => setError(String(cause)));
  }, []);

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

  function handleAddSite(input: string) {
    if (!state) return;
    void commit({
      ...state.rules,
      sites: [...state.rules.sites, { host: input.trim() }],
    });
  }

  function handleRemoveSite(host: string) {
    if (!state) return;
    void commit({
      ...state.rules,
      sites: state.rules.sites.filter((site) => site.host !== host),
    });
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
  }

  function handleRemoveWindow(id: string) {
    if (!state) return;
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

  return (
    <main className="mx-auto flex h-full max-w-2xl flex-col gap-5 p-6">
      <header className="flex flex-col gap-4">
        <h1 className="text-xl font-semibold text-ink">Break</h1>
        <Notice lines={NOTICE_LINES} />
        <nav className="flex gap-1 rounded-lg border border-line bg-surface-muted p-1">
          {TABS.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-current={tab === item.id ? "page" : undefined}
              onClick={() => setTab(item.id)}
              className={`flex-1 rounded-md px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent ${
                tab === item.id
                  ? "bg-surface text-ink"
                  : "text-ink-muted hover:text-ink"
              }`}
            >
              {item.label}
            </button>
          ))}
        </nav>
      </header>

      {state === null ? (
        <p className="text-sm text-ink-muted">규칙을 읽는 중입니다.</p>
      ) : (
        <>
          {tab === "status" && <StatusPanel state={state} />}
          {tab === "sites" && (
            <SiteList
              sites={state.rules.sites}
              onAdd={handleAddSite}
              onRemove={handleRemoveSite}
              error={error}
            />
          )}
          {tab === "schedule" && (
            <ScheduleEditor
              windows={state.rules.schedule.windows}
              onAdd={handleAddWindow}
              onRemove={handleRemoveWindow}
              error={error}
            />
          )}
        </>
      )}
    </main>
  );
}
