# DoH 차단 — 브라우저가 hosts를 우회하지 못하게

> 이슈: 없음
> 작업 종류: 기능 개발
> 상태: 완료
> 생성일: 2026-08-31

## 개요

`/etc/hosts`는 이름을 주소로 바꾸는 단계만 건드린다. Chromium 계열 브라우저는 "보안 DNS"가
기본값(자동)이면, 시스템 DNS가 자기가 아는 제공자일 때 DoH(DNS over HTTPS)로 승격해서 이름을
직접 해석한다. 그러면 OS 리졸버를 안 거치니 hosts가 통째로 무시된다.

이 Mac에서 실제로 재현됐다. Aside를 껐다 켜면 처음엔 막히다가, 검색 한 번으로 네트워크가 돌면
승격이 끝나면서 풀린다. `/etc/hosts` 블록은 그동안 그대로 붙어 있다.

macOS에 기본으로 들어 있는 PF 방화벽으로 DoH 연결 자체를 끊어서 막는다. DoH가 실패하면
Chromium은 시스템 리졸버로 되돌아가고, 그러면 hosts가 다시 먹는다.

**막을 대상은 목록으로 관리하지 않는다.** Chromium 자동 모드는 시스템 DNS와 같은 제공자로만
승격하므로, 시스템 네임서버 IP의 443만 막으면 충분하다. `scutil --dns`로 매번 읽으니 공유기가
DNS를 바꿔도 따라간다.

포함: PF 앵커 관리, `pf.conf` 연결, 차단 구간에만 적용, 해제 경로, 앱에 상태 노출
제외: 앱 차단(로드맵 1번), 잠금 모드, Windows

## Phase 1: 막을 대상과 규칙 문자열 계산

root 없이 검증되는 순수 함수만 만든다. `cargo test`로 끝난다.

- [x] 시스템 네임서버 IP 읽기 (`src/system/break-enforcer/src/pf/resolvers.rs`)
  - `scutil --dns` 출력에서 `nameserver[N] : <IP>` 줄을 뽑아 중복을 지운다
  - 파싱은 문자열을 받는 순수 함수로 두고, 명령 실행은 얇은 껍데기로 분리한다
  - 패턴: `browsers.rs`의 `running_executables` — 명령 실행과 파싱을 나눠 파싱만 테스트
- [x] PF 앵커 규칙 렌더링 (`src/system/break-enforcer/src/pf/rules.rs`)
  - 네임서버 IP마다 TCP/UDP 443 아웃바운드를 막는 `block drop out quick proto ...` 줄
  - IP가 없으면 빈 문자열 (hosts의 `render_block`과 같은 규약)
  - 재사용: `src/system/break-enforcer/src/hosts.rs`의 `render_block` 구조
- [x] 루프백은 막지 않는다 — `127.0.0.1` / `::1`이 네임서버로 잡히면 제외

## Phase 2: pf.conf에 앵커 걸기

- [x] `pf.conf` 마커 블록 삽입·제거 (`src/system/break-enforcer/src/pf/conf.rs`)
  - `anchor "com.movie42.break"` + `load anchor ... from "/etc/pf.anchors/com.movie42.break"` 두 줄
  - hosts와 같은 `# BEGIN Break` / `# END Break` 마커로 감싸 되돌리기가 한 함수가 되게 한다
  - 재사용: `hosts::strip_block`, `hosts::apply_to_content` — 로직이 같으므로 공용 함수로 올린다
- [x] 원자적 파일 쓰기를 공용으로 올림 (`src/system/break-enforcer/src/atomic.rs`)
  - 지금 `hosts.rs`의 `write_atomically`는 private이다. `pf.conf`와 앵커 파일도 같은 방식이 필요하다
  - 재사용: `hosts.rs`의 `write_atomically` 그대로 이동, `hosts.rs`는 새 모듈을 부르게 고침
- [x] 앵커 줄은 Apple 앵커 **뒤에** 온다 — `pf.conf` 주석이 메인 룰셋을 비우지 말라고 못박고 있다

## Phase 3: pfctl 실행부

- [x] 앵커 파일 쓰기와 로드 (`src/system/break-enforcer/src/pf/mod.rs`)
  - `/etc/pf.anchors/com.movie42.break`에 Phase 1 결과를 쓰고 `pfctl -a com.movie42.break -f <파일>`
  - 내용이 같으면 아무것도 하지 않는다 — hosts의 `read_and_write`가 `changed`를 돌려주는 것과 같은 규약
- [x] PF 켜기/끄기 참조 관리
  - `pfctl -E`가 돌려주는 토큰을 파일에 보관하고, 해제할 때 `pfctl -X <토큰>`
  - `pf.conf` 주석이 요구하는 방식이다. 그냥 `pfctl -d`를 부르면 다른 앱의 PF까지 내린다
  - 토큰 파일 위치: `/Library/Application Support/Break/pf.token`
- [x] dry-run 분기 — `is_dry_run()`이면 실행 대신 로그
  - 재사용: `src/system/break-enforcer/src/macos.rs`의 `is_dry_run` / `log_dry_run_sites` 패턴
- [x] 해제는 역순 — 앵커 비우기 → `pf.conf` 마커 제거 → 토큰 반납

## Phase 4: Enforcer와 데몬 연동

- [x] `Enforcer` trait에 두 메서드 추가 (`src/system/break-enforcer/src/lib.rs`)
  - `apply_dns_guard(&self) -> Result<(), Error>` / `clear_dns_guard(&self) -> Result<(), Error>`
  - `WindowsEnforcer`와 `UnsupportedEnforcer`는 `NotPrivileged` 스텁 — 지금 `apply_apps`와 같은 모양
- [x] `MacosEnforcer`에 구현 (`src/system/break-enforcer/src/macos.rs`)
- [x] 데몬이 hosts와 같은 tick에서 맞춤 (`src/system/break-daemon/src/reconcile.rs`)
  - 막을 사이트가 있으면 `apply_dns_guard`, 없으면 `clear_dns_guard`
  - `Outcome`에 DoH 상태를 실어 로그가 "차단 적용: youtube.com (DoH 차단)"처럼 나오게
  - 패턴: 지금 `apply`가 `sites.is_empty()`로 갈라지는 구조 그대로
- [x] `--clear`도 DoH 가드를 푼다 — 제거 스크립트가 이미 `--clear`를 부른다
  - 위치: `src/system/break-app/src/daemon/install.rs`의 `uninstall_script`

## Phase 5: 앱에 상태 노출

- [x] 설정 팝오버에 DoH 가드 한 줄 (`src/client/features/settings/SettingsPopover.tsx`)
  - 재사용: 방금 넣은 "설치된 것이 앱과 같습니다" 줄과 같은 자리·같은 톤
- [x] 적용 실패를 화면에 올림 (`src/system/break-app/src/commands.rs`, `AppState`)
  - 조용히 실패하지 않게 한다 — 이번 작업에서 두 번 물린 지점이다

## 진행 기록

### 2026-09-03

- 결정: `/etc/pf.conf` 두 줄은 차단 구간에만 넣는다 — 차단이 아닐 때는 순정 상태로 되돌린다
- 결정: DoH 가드 on/off 스위치는 두지 않는다 — hosts 차단이 먹으려면 사실상 필수라 끄는 선택지의 쓸모가 적다
- 결정: SelfControl 공존을 위한 별도 코드는 없다 — 매 tick의 `pf.conf` 확인·복구가 macOS 업데이트 경우와 같은 경로다
- 결정: 실패 문구는 상태 한 줄(`보안 DNS 차단: 실패`), 상세 원인은 데몬 로그에만
- Phase 1 완료
- 검증: `cargo test -p break-enforcer` 통과. 렌더된 규칙 문자열을 `pfctl -n -f`로 따로 파싱시켜 문법을 확인했다
- Phase 2 완료
- 결정: 마커 처리와 원자적 쓰기를 `marker.rs` / `atomic.rs`로 올렸다. `hosts.rs`는 두 모듈을 부르는 형태로 바뀌었고 기존 hosts 테스트 9개는 그대로 통과한다
- Phase 3 완료
- 결정: `pfctl -E`를 매 tick 부르면 참조 카운트가 새므로, 토큰 파일이 있으면 다시 부르지 않는다. 다만 재부팅하면 PF 상태는 초기화되는데 토큰 파일은 남으므로, 데몬 프로세스가 시작하고 처음 적용할 때는 옛 토큰을 반납하고 다시 얻는다
- 결정: 앱이 root 없이 상태를 알 수 있게 `/Library/Application Support/Break/pf.status`에 결과를 남긴다 (`ok` 또는 실패 메시지, 해제 시 파일 삭제). `pf.conf`만 읽으면 pfctl 호출이 실패한 경우를 적용 중으로 잘못 읽는다
- Phase 4 완료
- 검증: `cargo test --workspace` 통과 (17개 스위트), `cargo clippy --workspace --all-targets` 무경고
- 결정: `Outcome::Blocked`가 `Guard`를 함께 싣는다. DoH가 실패해도 hosts 차단은 `Blocked`로 남고 로그 뒤에 실패 사유가 붙는다
- Phase 5 완료
- 결정: 상태 줄은 `pf.status` 파일 유무로 판별한다. 차단 구간 여부로 판별하면 해제가 실패해 PF 규칙이 남았을 때 화면에 아무것도 안 뜬다
- 결정(계획 밖): `scutil --dns`가 빈 결과를 줄 때 `/etc/resolv.conf`를 폴백으로 읽는다. 네임서버를 못 구하면 기능 전체가 조용히 무효화되는데, 계획의 "조용히 실패하지 않게 한다"와 정면으로 어긋난다
- 검증: `cargo test --workspace` 통과, `cargo clippy --workspace --all-targets` 무경고, `bun run build`(tsc 포함)·`bun run lint` 통과
- 검증: `BREAK_DRY_RUN=1 break-daemon --once` → `dns guard: 1.0.0.3, 1.1.1.3` / `차단 적용: youtube.com (보안 DNS 차단)`, `--clear` → `차단 해제`
- 확인된 사실: 이 Mac의 시스템 DNS는 1.1.1.3 / 1.0.0.3 (Cloudflare). DoH 승격 조건에 해당하며 계획서의 진단과 맞는다
- 블로커: root 경로는 미검증이다. `pfctl -E` 토큰 파싱, `/etc/pf.conf` 실제 수정, 브라우저가 실제로 DoH를 포기하는지는 데몬을 설치해 돌려봐야 확인된다. 규칙 문자열 자체는 `pfctl -n -f`로 문법을 확인했다
- 전체 구현 완료

---

## 후속: hosts가 Chromium을 못 막는다 (2026-09-03)

> 작업 종류: 버그 수정
> 상태: 완료

Aside가 차단 구간에 새로 켜졌는데도 YouTube를 그대로 열었다. 측정해 보니 차단 시작(17:26:30)
이후 Aside가 새로 받아 캐시에 쓴 항목이 343개(`www.youtube.com` 81건, `googlevideo.com` 포함)였다.
캐시 항목은 네트워크 응답을 실제로 받아야 생긴다. OS 리졸버는 `www.youtube.com`을 `::1`로
막고 있었으므로, Chromium이 자체 DNS 클라이언트로 OS를 건너뛴 것이다. DoH를 막아도 평범한
DNS(53번)로 내려갈 뿐이라 이 경로는 hosts로 닫히지 않는다.

## 변경

- `src/system/break-enforcer/src/policy.rs` (신규) — `/Library/Managed Preferences/<번들ID>.plist`에
  `URLBlocklist`를 쓴다. 브라우저가 주소 단계에서 거부하므로 DNS·캐시·DoH와 무관하다
- 대상 탐지 — `/Applications`에서 Chromium 프레임워크(`*.framework/Versions/Current/Helpers`)가 있고
  `CFBundleURLTypes`에 http/https를 선언한 앱만. 프레임워크 조건만으로는 Slack·Discord·VSCode·
  Final Cut까지 잡힌다
- 남의 정책 파일은 건드리지 않는다 — plist에 `BreakManaged` 키를 넣고, 그 키가 없는 파일은 읽고 지나간다
- `Enforcer`에 `apply_browser_policy` / `clear_browser_policy` 추가, `reconcile`이 hosts와 같은 tick에서 맞춤
- 제거 스크립트가 `pf.token` / `pf.status`도 지운다

Safari는 OS 리졸버를 써서 hosts로 이미 막히므로 정책 대상이 아니다.

## 진행 기록

### 2026-09-03

- 검증: `cargo test --workspace` 통과(18개 스위트), `cargo clippy --workspace --all-targets` 무경고, `bun run build`·`bun run lint` 통과
- 검증: 생성한 plist를 `plutil -lint`로 확인, `plutil -extract URLBlocklist json`이 `["youtube.com"]`을 돌려줌
- 검증: dry-run → `browser policy: at.studio.AsideBrowser, com.google.Chrome, com.naver.Whale, company.thebrowser.Browser, company.thebrowser.dia → youtube.com`
- 결정: 제거 스크립트에서 `/Library/Managed Preferences/*.plist`를 통째로 지우려다 되돌렸다. MDM 등 남의 정책까지 지운다. `--clear`가 마커를 보고 우리 파일만 지운다
- 블로커: 새 데몬을 설치해 실제로 Aside가 막히는지는 확인하지 못했다. 앱에서 "다시 설치"가 필요하다
