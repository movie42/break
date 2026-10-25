import { useNow } from "@/shared/hooks/useNow";
import type { InstantBlock, TimeWindow } from "@/shared/types";
import { WEEKDAY_LABELS, WEEKDAYS } from "@/shared/types";
import { SectionLabel } from "@/shared/ui/SectionLabel";

import { formatClock, instantEndsAt } from "./active";
import type { Span } from "./merge";
import { mergedByDay } from "./merge";
import { MINUTES_PER_DAY, formatMinutes } from "./time";

const RULER_HOURS = [0, 6, 12, 18, 24];
const TICK_HOURS = [6, 12, 18];
const TRACK_HEIGHT = 14;
const TICK_WIDTH = 2;
const MARKER_RADIUS = 5;
const LABEL_SPACE = 14;
const CLOCK_MS = 1000;

interface WeekTimelineProps {
  windows: TimeWindow[];
  instant: InstantBlock | null;
}

function instantByDay(instant: InstantBlock | null, now: Date): Span[][] {
  const byDay: Span[][] = WEEKDAYS.map(() => []);
  const ends = instantEndsAt(instant, now);
  if (ends === null) {
    return byDay;
  }

  const today = (now.getDay() + 6) % 7;
  const midnight = new Date(now);
  midnight.setHours(0, 0, 0, 0);

  for (let day = today; day < WEEKDAYS.length; day++) {
    const dayStart = new Date(midnight);
    dayStart.setDate(midnight.getDate() + (day - today));
    const from = Math.max(now.getTime(), dayStart.getTime());
    const to = Math.min(
      ends.getTime(),
      dayStart.getTime() + MINUTES_PER_DAY * 60_000,
    );
    if (to <= from) {
      break;
    }
    byDay[day].push({
      start: (from - dayStart.getTime()) / 60_000,
      end: (to - dayStart.getTime()) / 60_000,
    });
  }
  return byDay;
}

interface NowMarkerProps {
  now: Date;
}

function NowMarker({ now }: NowMarkerProps) {
  const minutes =
    now.getHours() * 60 + now.getMinutes() + now.getSeconds() / 60;
  const position = `${(minutes / MINUTES_PER_DAY) * 100}%`;
  const centerY = LABEL_SPACE + TRACK_HEIGHT / 2;

  return (
    <svg
      className="pointer-events-none absolute inset-x-0 -top-3.5 h-7 w-full overflow-visible"
      aria-hidden="true"
    >
      <svg x={position} y={0} overflow="visible">
        <text
          x={0}
          y={LABEL_SPACE - 4}
          textAnchor="middle"
          className="fill-ink text-[9.5px] font-semibold tabular-nums"
        >
          {formatClock(now)}
        </text>
        <circle
          cx={0}
          cy={centerY}
          r={MARKER_RADIUS}
          strokeWidth={2}
          className="fill-ink stroke-surface"
        />
      </svg>
    </svg>
  );
}

function rulerOffset(hour: number): string {
  if (hour === 0) {
    return "left-0";
  }
  if (hour === 24) {
    return "right-0";
  }
  return hour === 6
    ? "left-1/4 -translate-x-1/2"
    : hour === 12
      ? "left-1/2 -translate-x-1/2"
      : "left-3/4 -translate-x-1/2";
}

export function WeekTimeline({ windows, instant }: WeekTimelineProps) {
  const now = useNow(CLOCK_MS);
  const onByDay = mergedByDay(windows.filter((window) => window.enabled));
  const offByDay = mergedByDay(windows.filter((window) => !window.enabled));
  const instantSpans = instantByDay(instant, now);
  const today = (now.getDay() + 6) % 7;

  return (
    <section className="rounded-xl border border-line bg-surface px-4.5 pt-4 pb-4 shadow-card">
      <div className="mb-3 flex items-baseline justify-between">
        <SectionLabel>실제로 차단되는 시간</SectionLabel>
        <span className="text-[11px] text-ink-faint">
          켜 둔 시간대만 그립니다. 흐린 구간은 꺼 둔 시간대입니다
        </span>
      </div>

      <div className="relative mb-1.5 ml-[26px] h-3">
        {RULER_HOURS.map((hour) => (
          <span
            key={hour}
            className={`absolute top-0 text-[10px] text-ink-faint ${rulerOffset(hour)}`}
          >
            {hour}
          </span>
        ))}
      </div>

      <div className="flex flex-col gap-[3px]">
        {WEEKDAYS.map((day, index) => {
          const spans = [...onByDay[index], ...instantSpans[index]];
          const idle = offByDay[index];
          const label = WEEKDAY_LABELS[day];
          const summary =
            spans.length === 0
              ? "차단 없음"
              : spans
                  .map(
                    (span) =>
                      `${formatMinutes(span.start)}—${formatMinutes(span.end)}`,
                  )
                  .join(", ");

          return (
            <div
              key={day}
              className={`flex items-center gap-2.5 ${index === today ? "pt-3.5" : ""}`}
            >
              <span
                className={`w-4 text-center text-[11px] font-medium ${
                  index === today
                    ? "font-bold text-accent"
                    : index > 4
                      ? "text-ink-faint"
                      : "text-ink-muted"
                }`}
              >
                {label}
              </span>
              <div className="relative h-3.5 flex-1">
                <div className="h-3.5 overflow-hidden rounded bg-surface-muted">
                  <svg
                    className="block h-3.5 w-full"
                    viewBox={`0 0 ${MINUTES_PER_DAY} ${TRACK_HEIGHT}`}
                    preserveAspectRatio="none"
                    role="img"
                    aria-label={`${label}요일 ${summary}`}
                  >
                    {TICK_HOURS.map((hour) => (
                      <rect
                        key={hour}
                        className="fill-line"
                        x={hour * 60}
                        y={0}
                        width={TICK_WIDTH}
                        height={TRACK_HEIGHT}
                      />
                    ))}
                    {idle.map((span) => (
                      <rect
                        key={`off-${span.start}-${span.end}`}
                        className="fill-line-strong"
                        x={span.start}
                        y={0}
                        width={span.end - span.start}
                        height={TRACK_HEIGHT}
                      />
                    ))}
                    {spans.map((span) => (
                      <rect
                        key={`on-${span.start}-${span.end}`}
                        className="fill-accent"
                        x={span.start}
                        y={0}
                        width={span.end - span.start}
                        height={TRACK_HEIGHT}
                      />
                    ))}
                  </svg>
                </div>
                {index === today && <NowMarker now={now} />}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
