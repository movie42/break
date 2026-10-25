import { useEffect, useState } from "react";

import { recentEvents } from "@/shared/api/rules";
import type { BreakEvent } from "@/shared/types";

const REFRESH_MS = 5000;

export function useRecentEvents(): BreakEvent[] {
  const [events, setEvents] = useState<BreakEvent[]>([]);

  useEffect(() => {
    let alive = true;
    function load() {
      recentEvents()
        .then((next) => {
          if (alive) setEvents(next);
        })
        .catch(() => {});
    }
    load();
    const timer = window.setInterval(load, REFRESH_MS);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, []);

  return events;
}
