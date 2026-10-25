import { useState } from "react";

import { formatCountdown, formatReleaseAt } from "@/features/schedule/active";
import { useNow } from "@/shared/hooks/useNow";
import type { MailAccount, Rules } from "@/shared/types";
import { Button } from "@/shared/ui/Button";
import { SectionLabel } from "@/shared/ui/SectionLabel";

const COUNTDOWN_MS = 1000;
const GMAIL_HOST = "smtp.gmail.com";
const SMTPS_PORT = 465;

const INPUT_CLASS =
  "h-9 rounded-lg border border-line-strong bg-surface px-3 text-[13px] text-ink outline-none placeholder:text-ink-faint focus-visible:border-accent focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent";

const CARD_CLASS =
  "mb-4 rounded-xl border border-line bg-surface px-4.5 pt-4 pb-4.5 shadow-card";

interface ProtectTabProps {
  rules: Rules;
  mailAccount: string | null;
  guarded: boolean;
  releaseDelaySeconds: number;
  onRequestOff: (password: string) => void;
  onCancelOff: () => void;
  onEnable: () => void;
  onAddRecipient: (email: string) => void;
  onRequestRemoval: (email: string, password: string) => void;
  onCancelRemoval: (email: string) => void;
  onSetAccount: (account: MailAccount, password: string | null) => void;
}

function releaseTime(requestedAt: string, delaySeconds: number): Date {
  return new Date(new Date(requestedAt).getTime() + delaySeconds * 1000);
}

function describeDelay(seconds: number): string {
  return seconds % 3600 === 0
    ? `${seconds / 3600}시간`
    : `${Math.round(seconds / 60)}분`;
}

interface CountdownProps {
  requestedAt: string;
  delaySeconds: number;
  label: string;
  onCancel: () => void;
}

function Countdown({
  requestedAt,
  delaySeconds,
  label,
  onCancel,
}: CountdownProps) {
  const now = useNow(COUNTDOWN_MS);
  const at = releaseTime(requestedAt, delaySeconds);
  return (
    <div className="flex items-center justify-between rounded-lg border border-accent-line bg-accent-soft px-3.5 py-2.5">
      <div className="flex flex-col">
        <span className="text-[13px] font-semibold text-ink">
          {formatReleaseAt(now, at)}에 {label}
        </span>
        <span className="text-[11.5px] text-ink-muted tabular-nums">
          남은 시간 {formatCountdown(now, at)}
        </span>
      </div>
      <Button variant="secondary" onClick={onCancel}>
        취소
      </Button>
    </div>
  );
}

interface AccountFormProps {
  current: string | null;
  onSave: (account: MailAccount, password: string | null) => void;
}

function AccountForm({ current, onSave }: AccountFormProps) {
  const [open, setOpen] = useState(current === null);
  const [username, setUsername] = useState("");
  const [secret, setSecret] = useState("");
  const [host, setHost] = useState(GMAIL_HOST);
  const [password, setPassword] = useState("");

  const replacing = current !== null;
  const valid =
    username.includes("@") &&
    secret.length > 0 &&
    host.length > 0 &&
    (!replacing || password.length > 0);

  if (!open) {
    return (
      <div className="flex items-center justify-between">
        <span className="text-[12.5px] text-ink-muted">
          보내는 계정: {current}
        </span>
        <Button variant="secondary" onClick={() => setOpen(true)}>
          바꾸기
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      <p className="text-[11.5px] leading-relaxed text-ink-faint">
        알림은 이 계정으로 보냅니다. Gmail은 일반 비밀번호로 보낼 수 없어서,
        Google 계정 설정에서 만든 앱 비밀번호를 넣어야 합니다.
      </p>
      <input
        value={username}
        placeholder="보내는 메일 주소"
        aria-label="보내는 메일 주소"
        onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
          setUsername(event.target.value)
        }
        className={INPUT_CLASS}
      />
      <input
        type="password"
        value={secret}
        placeholder="앱 비밀번호"
        aria-label="앱 비밀번호"
        onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
          setSecret(event.target.value)
        }
        className={INPUT_CLASS}
      />
      <input
        value={host}
        placeholder="메일 서버"
        aria-label="메일 서버"
        onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
          setHost(event.target.value)
        }
        className={INPUT_CLASS}
      />
      {replacing && (
        <input
          type="password"
          value={password}
          placeholder="Break 비밀번호"
          aria-label="Break 비밀번호"
          onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
            setPassword(event.target.value)
          }
          className={INPUT_CLASS}
        />
      )}
      <div className="flex justify-end gap-2">
        {replacing && (
          <Button variant="quiet" onClick={() => setOpen(false)}>
            취소
          </Button>
        )}
        <Button
          disabled={!valid}
          onClick={() =>
            onSave(
              {
                host: host.trim(),
                port: SMTPS_PORT,
                username: username.trim(),
                secret,
                from: username.trim(),
              },
              replacing ? password : null,
            )
          }
        >
          저장
        </Button>
      </div>
    </div>
  );
}

export function ProtectTab({
  rules,
  mailAccount,
  guarded,
  releaseDelaySeconds,
  onRequestOff,
  onCancelOff,
  onEnable,
  onAddRecipient,
  onRequestRemoval,
  onCancelRemoval,
  onSetAccount,
}: ProtectTabProps) {
  const [offPassword, setOffPassword] = useState("");
  const [email, setEmail] = useState("");
  const [removing, setRemoving] = useState<string | null>(null);
  const [removePassword, setRemovePassword] = useState("");
  const filter = rules.adultFilter;

  return (
    <>
      <section className={CARD_CLASS}>
        <div className="mb-3 flex items-baseline justify-between">
          <SectionLabel>유해사이트 차단</SectionLabel>
          <span
            className={`text-[11.5px] font-semibold ${filter.enabled ? "text-ok" : "text-danger"}`}
          >
            {filter.enabled ? "켜짐" : "꺼짐"}
          </span>
        </div>
        {!filter.enabled ? (
          <div className="flex items-center justify-between">
            <span className="text-[12.5px] text-ink-muted">
              성인·도박 사이트가 막히지 않고 있습니다.
            </span>
            <Button onClick={onEnable}>다시 켜기</Button>
          </div>
        ) : filter.disableRequestedAt !== null ? (
          <Countdown
            requestedAt={filter.disableRequestedAt}
            delaySeconds={releaseDelaySeconds}
            label="꺼집니다"
            onCancel={onCancelOff}
          />
        ) : (
          <div className="flex items-center gap-2">
            <input
              type="password"
              value={offPassword}
              placeholder="비밀번호"
              aria-label="비밀번호"
              onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                setOffPassword(event.target.value)
              }
              className={`${INPUT_CLASS} flex-1`}
            />
            <Button
              variant="danger"
              disabled={offPassword.length === 0}
              onClick={() => {
                onRequestOff(offPassword);
                setOffPassword("");
              }}
            >
              끄기 신청
            </Button>
          </div>
        )}
        <p className="mt-2.5 text-[11.5px] leading-relaxed text-ink-faint">
          끄기를 신청하면 {describeDelay(releaseDelaySeconds)} 뒤에 꺼집니다. 그
          전에는 언제든 취소할 수 있습니다.
          {!guarded &&
            " 차단 프로그램을 설치하기 전에는 규칙 파일을 직접 고쳐 이 대기를 건너뛸 수 있습니다."}
        </p>
      </section>

      <section className={CARD_CLASS}>
        <div className="mb-3">
          <SectionLabel>알림 받을 메일</SectionLabel>
        </div>
        <ul className="mb-3 flex list-none flex-col gap-2 p-0">
          {rules.alerts.recipients.map((recipient) => (
            <li key={recipient.email} className="flex flex-col gap-1.5">
              {recipient.removalRequestedAt !== null ? (
                <Countdown
                  requestedAt={recipient.removalRequestedAt}
                  delaySeconds={releaseDelaySeconds}
                  label={`${recipient.email}가 빠집니다`}
                  onCancel={() => onCancelRemoval(recipient.email)}
                />
              ) : removing === recipient.email ? (
                <div className="flex items-center gap-2">
                  <span className="min-w-0 flex-1 truncate text-[13px] text-ink">
                    {recipient.email}
                  </span>
                  <input
                    type="password"
                    value={removePassword}
                    placeholder="비밀번호"
                    aria-label="비밀번호"
                    autoFocus
                    onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                      setRemovePassword(event.target.value)
                    }
                    className={`${INPUT_CLASS} w-32`}
                  />
                  <Button
                    variant="danger"
                    disabled={removePassword.length === 0}
                    onClick={() => {
                      onRequestRemoval(recipient.email, removePassword);
                      setRemoving(null);
                      setRemovePassword("");
                    }}
                  >
                    빼기 신청
                  </Button>
                  <Button variant="quiet" onClick={() => setRemoving(null)}>
                    취소
                  </Button>
                </div>
              ) : (
                <div className="flex items-center justify-between">
                  <span className="text-[13px] text-ink">
                    {recipient.email}
                  </span>
                  <Button
                    variant="quiet"
                    onClick={() => setRemoving(recipient.email)}
                  >
                    지우기
                  </Button>
                </div>
              )}
            </li>
          ))}
        </ul>
        <div className="mb-2.5 flex items-center gap-2">
          <input
            value={email}
            placeholder="지켜봐 줄 사람의 메일 주소"
            aria-label="새 주소"
            onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
              setEmail(event.target.value)
            }
            className={`${INPUT_CLASS} flex-1`}
          />
          <Button
            disabled={!email.includes("@")}
            onClick={() => {
              onAddRecipient(email);
              setEmail("");
            }}
          >
            추가
          </Button>
        </div>
        <p className="mb-4 text-[11.5px] leading-relaxed text-ink-faint">
          추가는 바로 됩니다. 지우기는 비밀번호 +{}
          {describeDelay(releaseDelaySeconds)} 뒤에 반영되고, 그사이 모든 주소에
          미리 알립니다. 다른 사람 주소를 넣으면 그 주소로 확인 메일이 먼저
          갑니다. 차단 프로그램을 지우고 다시 켜지 않으면 알림이 가지 않습니다.
        </p>
        <div className="border-t border-surface-muted pt-3.5">
          <AccountForm current={mailAccount} onSave={onSetAccount} />
        </div>
      </section>
    </>
  );
}
