import { useState } from "react";

import { Button } from "@/shared/ui/Button";
import { SectionLabel } from "@/shared/ui/SectionLabel";

const MIN_PASSWORD = 4;

const INPUT_CLASS =
  "h-10 w-full rounded-lg border border-line-strong bg-surface px-3.5 text-sm text-ink outline-none placeholder:text-ink-faint focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent";

interface ConsentScreenProps {
  needsPassword: boolean;
  pending: boolean;
  error: string | null;
  onQuit: () => void;
  onAgree: (password: string | null) => void;
}

export function ConsentScreen({
  needsPassword,
  pending,
  error,
  onQuit,
  onAgree,
}: ConsentScreenProps) {
  const [step, setStep] = useState<"warn" | "password">("warn");
  const [password, setPassword] = useState("");
  const [repeat, setRepeat] = useState("");

  const tooShort = password.length < MIN_PASSWORD;
  const mismatch = repeat.length > 0 && repeat !== password;
  const ready = !tooShort && repeat === password;

  function handleAgree() {
    if (needsPassword) {
      setStep("password");
      return;
    }
    onAgree(null);
  }

  return (
    <div className="absolute inset-0 z-30 flex items-center justify-center bg-bg p-6">
      <section
        role="dialog"
        aria-modal="true"
        aria-label="유해사이트 차단 동의"
        className="w-[440px] rounded-2xl border border-accent-line bg-surface px-7 pt-7 pb-6 shadow-sheet"
      >
        {step === "warn" ? (
          <>
            <p className="mb-1 text-[11px] font-bold tracking-[0.12em] text-accent">
              처음 시작하기 전에
            </p>
            <p className="mb-4 text-[17px] font-bold text-ink">
              Break는 성인·도박 사이트를 항상 차단합니다
            </p>
            <p className="mb-2 text-[13px] leading-relaxed text-ink-muted">
              약 8만 개 사이트가 막힙니다. 끄려면 비밀번호와 긴 절차가
              필요합니다.
            </p>
            <p className="mb-6 text-[13px] leading-relaxed text-ink-muted">
              이 앱은 끄고 켠 기록을 남깁니다.
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="quiet" onClick={onQuit}>
                종료
              </Button>
              <Button onClick={handleAgree} disabled={pending}>
                동의하고 시작
              </Button>
            </div>
          </>
        ) : (
          <>
            <p className="mb-4 text-[17px] font-bold text-ink">
              비밀번호 정하기
            </p>
            <p className="mb-4 text-[12.5px] leading-relaxed text-ink-muted">
              유해사이트 차단을 끄거나 알림 메일 주소를 뺄 때 이 비밀번호를
              묻습니다. 정한 뒤에는 바꿀 수 없습니다.
            </p>
            <div className="mb-2">
              <SectionLabel>비밀번호</SectionLabel>
            </div>
            <input
              type="password"
              value={password}
              aria-label="비밀번호"
              autoFocus
              onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                setPassword(event.target.value)
              }
              className={`${INPUT_CLASS} mb-3`}
            />
            <div className="mb-2">
              <SectionLabel>한 번 더</SectionLabel>
            </div>
            <input
              type="password"
              value={repeat}
              aria-label="비밀번호 확인"
              onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                setRepeat(event.target.value)
              }
              className={`${INPUT_CLASS} mb-2`}
            />
            <p className="mb-5 min-h-4 text-[11.5px] text-ink-faint">
              {mismatch
                ? "두 비밀번호가 다릅니다"
                : tooShort
                  ? `${MIN_PASSWORD}자 이상으로 정하세요`
                  : ""}
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="quiet" onClick={() => setStep("warn")}>
                뒤로
              </Button>
              <Button
                onClick={() => onAgree(password)}
                disabled={!ready || pending}
              >
                정하고 시작
              </Button>
            </div>
          </>
        )}
        {error !== null && (
          <p className="mt-4 text-[12.5px] text-danger">{error}</p>
        )}
      </section>
    </div>
  );
}
