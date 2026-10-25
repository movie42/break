# 차단 계층 정리 — 해제가 안 되는 버그와 브라우저 정책의 존폐

> 이슈: 없음
> 작업 종류: 버그 수정
> 상태: 완료
> 생성일: 2026-09-03
> 참조: `docs/doh-block/plans/tasks.md` (완료), `docs/site-blocking/plans/tasks.md` (완료)

## 개요

지금 사이트 하나를 막을 때 세 겹이 동시에 걸린다. `/etc/hosts`, PF 앵커(DoH 승격 차단),
그리고 Chromium 계열 브라우저의 관리 정책(`URLBlocklist`)이다.

셋째 겹에서 해제가 동작하지 않는다. `policy.rs`는 `/Library/Managed Preferences/<번들ID>.plist`를
지우고 끝내는데, macOS는 그 값을 root로 도는 `cfprefsd` 프로세스 안에 캐시로 들고 있다. 파일이
사라져도 캐시는 그대로라, 브라우저를 껐다 켜도 계속 막힌다. 2026-09-03에 실제로 겪었고
`sudo killall cfprefsd`로만 풀렸다. 차단 도구에서 "걸리는데 안 풀리는" 실패는 "가끔 새는"
실패보다 나쁘다.

동시에 셋째 겹이 필요한지 자체가 검증된 적이 없다. PF DoH 가드와 브라우저 정책은 같은 커밋
(`ac921d0`)에서 함께 들어왔다. PF 가드만으로 hosts가 다시 먹는지 따로 재본 기록이 없다.
참고로 SelfControl(GPL)은 hosts + PF IP 차단만 쓰고 브라우저 정책은 안 쓴다.

이 작업은 순서가 핵심이다. **먼저 해제를 고쳐서 위험을 없애고, 그 다음 계층별로 재서 셋째 겹을
지울지 정한다.** 재보기 전에 지우면 "왜 안 막히지"를 다시 겪고, 그대로 두면 "왜 안 풀리지"를
또 겪는다.

포함: 정책 해제 경로 수정, 계층별 실측, 실측 결과에 따른 `policy.rs` 존폐 결정
제외: 앱 차단, 잠금 모드, PF IP 차단 도입(실측에서 필요하다고 나오면 별도 작업으로 뗀다)

## Phase 1: 정책 해제가 브라우저까지 도달하게 한다

여기까지가 오늘 겪은 버그의 수정이다. 계층 구성은 그대로 두고 해제 경로만 고친다.

- [x] 파일이 실제로 바뀌었는지 알려주도록 고친다 (`src/system/break-enforcer/src/policy.rs`)
  - 지금 `write_one`은 `io::Result<()>`라 "썼다/지웠다"와 "그대로였다"를 구분 못 한다. `bool`을 돌려주게 바꾸고 `write_all`이 그걸 모아서 올린다
  - 패턴: `hosts::read_and_write`가 이미 `Ok(bool)`로 변경 여부를 돌려주고, `macos.rs`가 그걸 보고 DNS 캐시를 비운다. 같은 모양으로 맞춘다
- [x] 설정 캐시를 비우는 함수 (`src/system/break-enforcer/src/policy.rs`)
  - `killall cfprefsd`를 실행한다. 데몬은 root라 root/사용자 것 모두 죽는다. launchd가 다시 띄우므로 되돌릴 것이 없다
  - 재사용: `macos.rs::flush_dns_cache`가 정확히 같은 모양(명령 실행, 결과 무시, 실패해도 진행). 바로 옆에 나란히 둔다
  - `is_dry_run()`이면 실행하지 않는다
- [x] 정책 파일이 바뀐 tick에서만 캐시를 비운다 (`src/system/break-enforcer/src/policy.rs`)
  - 데몬은 1초마다 돈다. 매 tick `killall`을 부르면 시스템 전체 설정 읽기가 계속 흔들린다
- [x] 회귀 테스트 (`src/system/break-enforcer/tests/browser_policy.rs`)
  - 같은 내용을 두 번 쓰면 두 번째는 "바뀌지 않음"이 나온다
  - 남이 만든 정책 파일(`BreakManaged` 키 없음)은 건드리지도, "바뀜"으로 세지도 않는다
- [x] 정책 값이 살아 있는지 확인하는 검사 도구 (`scripts/policy-probe.py`)
  - Chrome이 정책을 읽는 것과 같은 경로(`CFPreferencesCopyAppValue` / `CFPreferencesAppValueIsForced`)로 물어본다. 파일 유무가 아니라 **브라우저가 실제로 보는 값**을 봐야 한다
  - `defaults read`로는 안 된다 — 이번에 관리 정책이 살아 있는데도 "does not exist"가 나왔다
  - Phase 2 실측에서도 계속 쓴다

검증: `cargo test` 통과. 그리고 실제로 차단을 걸었다 푼 뒤 `scripts/policy-probe.py`가
`forced=False value=None`을 내야 한다.

## Phase 2: 계층별로 재서 셋째 겹이 필요한지 정한다

계측용 스위치를 넣고, 계층을 하나씩 끄면서 같은 조건으로 세 번 잰다.

- [x] 계층을 끄는 환경변수 (`src/system/break-enforcer/src/lib.rs`)
  - `BREAK_SKIP_LAYERS=policy,dns` 형태. 이름이 들어간 계층은 apply를 건너뛴다. **clear는 절대 건너뛰지 않는다** — 계측 때문에 해제가 막히면 안 된다
  - 패턴: `DRY_RUN_ENV`(`BREAK_DRY_RUN`)가 이미 같은 방식으로 환경변수를 읽는다. 그 옆에 둔다
  - 데몬 plist에 환경변수를 넣는 방법을 `install.rs`에서 확인해 실측 절차에 적는다
- [x] 실측 A — hosts만 (`docs/block-layers/plans/tasks.md` 진행 기록)
  - `BREAK_SKIP_LAYERS=policy,dns`. 브라우저를 완전히 종료 후 새로 켜고 `youtube.com`
- [x] 실측 B — hosts + PF DoH 가드 (정책만 끔)
  - `BREAK_SKIP_LAYERS=policy`. 이게 SelfControl과 같은 수준이고, 이 작업의 핵심 질문이다
- [x] 실측 C — 세 겹 전부 (지금 상태, 대조군)
- [x] 각 실측을 브라우저를 켜 둔 채로도 한 번 더 잰다
  - 커밋 `ac921d0`의 관찰("차단 중 Aside가 YouTube를 열었다")이 DoH 때문인지 브라우저가 들고 있던 DNS 캐시·연결 때문인지 여기서 갈린다
- [x] 결과를 표로 `spec.md`에 적는다 — 브라우저 / 계층 / 껐다 켬 여부 / 열렸는지

검증: 표의 칸이 전부 채워진다. Chrome과 Aside 둘 다, Safari는 대조군으로 한 번.

## Phase 3: 결과대로 정리한다

Phase 2의 실측 B가 답을 정한다. 어느 쪽이든 계측 스위치는 지운다.

- [~] 실측 B가 "안 샘"이면 브라우저 정책 계층을 들어낸다 — 해당 없음 (B에서 Aside가 샘)
  - `src/system/break-enforcer/src/policy.rs` 삭제, `lib.rs`의 모듈 선언 제거
  - `Enforcer` 트레이트에서 `apply_browser_policy` / `clear_browser_policy` 제거 (`lib.rs`, `macos.rs`, `windows.rs`, `unsupported`)
  - `reconcile.rs`의 호출과 `Guard` 로그 문구("브라우저 정책 + 보안 DNS 차단") 정리
  - `tests/browser_policy.rs` 삭제, `reconcile.rs`의 `FakeEnforcer`에서 해당 메서드 제거
- [x] 실측 B가 "샘"이면 정책을 남기고 Phase 1 수정만 유지한다
  - 남기는 이유를 실측 표와 함께 `spec.md`에 적는다. 다음에 또 "이거 왜 있지"가 나오지 않도록
- [x] 계측 스위치 제거 (`src/system/break-enforcer/src/lib.rs`)
- [x] 문서 갱신 (`docs/roadmap.md`, `docs/doh-block/plans/spec.md`)
  - roadmap의 우회 방지 표에서 계층 구성이 바뀌면 그 줄을 고친다

검증: `cargo test`, `cargo clippy --all-targets` 무경고. 실측 B와 같은 조건으로 한 번 더 걸었다
풀어서 정상 동작 확인.

## 진행 기록

### 2026-09-03

- 계획 작성
- 결정: Phase 2 실측은 사용자가 나중에 직접 돌린다. 코드와 절차를 먼저 다 만들어 두고, 결과가 오면 Phase 3을 이어간다
- 결정: 실측 B가 "안 샘"이면 `policy.rs`를 지운다. 계층이 줄면 이번 해제 실패 경로도 같이 사라진다
- Phase 1 완료
- 검증: `cargo test` 123개 통과, `cargo clippy --all-targets` 무경고. `python3 scripts/policy-probe.py`가 Chromium 5종에 대해 `forced=False value=None`을 냈다 (차단이 걸리지 않은 상태 기준선)
- 결정: 캐시 플러시(`flush_preferences_cache`)는 `policy.rs`가 아니라 `macos.rs`의 `flush_dns_cache` 옆에 뒀다. hosts 계층과 같은 모양 — 모듈은 변경 여부만 돌려주고 플러시는 `macos.rs`가 판단한다
- 결정: 계측 스위치를 데몬 plist에 넣지 않았다. `plist.rs`에 `EnvironmentVariables`를 추가하면 계측용 키가 배포본에 남는다. 대신 launchd 데몬을 내리고 같은 바이너리를 터미널에서 직접 띄우는 절차로 적었다
- 블로커: Phase 2 실측 A~C는 사용자가 브라우저를 껐다 켜며 직접 재야 한다. 절차는 `spec.md` → 실측 절차
- 배경: 2026-09-03 18:10~18:58 사이 재현. 정책 파일은 지워졌는데 `CFPreferencesCopyAppValue`가
  `URLBlocklist=["youtube.com"]`, `forced=True`를 계속 돌려줬다. `sudo killall cfprefsd`
  (root PID 402 → 63360 재시작) 이후 `forced=False`로 바뀌었다
- SelfControl 소스 확인: `BlockManager.m`이 도메인마다 hosts 한 줄 + PF에 IP 차단 한 줄을 넣는다.
  단 구글 도메인은 IP 공유 때문에 PF에서 제외하고 hosts에만 의존한다(`rely on the domain-level
  blocking instead`). IPv6은 PF로 안 막는다. 즉 YouTube는 그쪽도 hosts만으로 막는다

- Phase 2 완료 (실측: 사용자 실행)
- 결과: 실측 B(hosts + PF)에서 Chrome·Safari는 막히고 **Aside만 뚫렸다**. 강력 새로고침하면 열리고, 차단 중 Aside가 `tv-in-f95.1e100.net:443`(YouTube 실서버)에 TCP 연결을 유지하고 있었다
- 결과: 실측 C(세 겹 전부)에서 Aside가 "조직에서 차단했습니다"로 막혔다
- 실측 A는 재지 않았다. B와 C로 결론이 갈려서 hosts 단독 성능은 판단에 영향이 없다
- 검증: 실측 C 해제 후 "조직에서 차단" 화면이 즉시 풀렸고 `policy-probe.py --expect-clear` 통과. Phase 1에서 고친 `cfprefsd` 캐시 플러시가 실제 환경에서 동작하는 것을 확인했다
- Phase 3 완료
- 결정: `policy.rs`를 남긴다. 세 겹 중 Aside를 막는 것은 브라우저 정책뿐이다. 근거는 `spec.md` → 결론
- 결정: `docs/roadmap.md`의 "DoH 쓰는 브라우저 → PF 방화벽" 줄을 "이미 막힘 (브라우저 정책)"으로 고쳤다. 실측에서 PF만으로는 Aside를 못 막았다
- 검증: `cargo test` 119개 통과, `cargo clippy --all-targets` 무경고 (계측 스위치 제거 후)
- 남은 의문: Chrome과 Aside 모두 DoH 설정이 비어 있는데 Aside만 hosts를 지나간다. 경로를 특정하지 못했다. `docs/roadmap.md` → 아직 확인 못 한 것에 적어 뒀다
- 검증: 앱 경로 확인 (`bun run tauri dev` → 시간대 추가 → 데몬 설치 → 차단 → 해제). launchd로 뜬 데몬에서도 해제가 브라우저까지 닿았다. 버그가 처음 난 경로가 이쪽이라 이걸로 재현 조건을 다 덮었다
- 전체 구현 완료
