import { describeEvent, latestDowntime } from "./describe";
import { useRecentEvents } from "./useRecentEvents";

export function DowntimeBanner() {
  const events = useRecentEvents();
  const down = latestDowntime(events, new Date());
  if (down === null) {
    return null;
  }

  return (
    <section className="mb-4 rounded-xl border border-accent-line bg-accent-soft px-4.5 py-3">
      <p className="text-[13px] font-bold text-ink">! {describeEvent(down)}</p>
      <p className="text-[11.5px] text-ink-muted">
        그동안은 사이트가 막히지 않았습니다. 알림 메일 주소가 있으면 그 주소로도
        알렸습니다.
      </p>
    </section>
  );
}
