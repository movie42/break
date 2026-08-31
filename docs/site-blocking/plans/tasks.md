# 사이트 차단

> 이슈: 없음
> 작업 종류: 기능 개발
> 상태: 완료
> 생성일: 2026-08-31

## 개요

초기 셋업(`docs/initial-setup/plans/`)에서 규칙을 편집해 파일로 저장하는 것까지 만들었다. 저장은 되지만 아무것도 차단하지 않는다. 이번 작업에서 그 규칙을 실제 차단으로 바꾼다.

차단 방식은 `hosts` 파일이다. 운영체제가 도메인 이름을 IP 주소로 바꿀 때 가장 먼저 읽는 파일이고, 여기에 `127.0.0.1 youtube.com`을 적어두면 브라우저가 자기 컴퓨터로 연결을 시도하다 실패한다. 이 파일은 root 소유라 로그인한 사용자로 뜬 GUI는 쓰기가 거부된다. 그래서 root로 도는 상주 프로세스(`break-daemon`)를 따로 설치하고, GUI는 규칙 파일만 쓴다.

역할 분담은 이렇게 나눈다.

| | 하는 일 | 권한 |
| --- | --- | --- |
| GUI (`break-app`) | 규칙 편집·저장, 데몬 설치·제거 요청, 상태 표시 | 로그인 사용자 |
| 데몬 (`break-daemon`) | 규칙 파일을 주기적으로 읽고 지금이 차단 구간이면 `hosts`를 맞춘다 | root |
| `break-enforcer` | `hosts` 파일을 실제로 고치는 코드 | 호출한 쪽의 권한을 따름 |

**이번 작업의 성패는 "앱에서 사이트와 시간대를 넣으면 그 시간에 브라우저에서 실제로 안 열린다"까지 확인되느냐다.**

- 포함: `hosts` 갱신 구현, 데몬 상주 루프, LaunchDaemon 설치·제거, GUI의 데몬 상태 화면, DoH 우회 여부 확인
- 제외: 앱 차단(다음 작업), Windows 구현(다음 작업), 우회 방지 장치, 앱 번들 배포·코드 서명

### 이번 작업에서 확정된 방식

- **권한 획득**: GUI가 `osascript`로 관리자 암호를 한 번 받아 데몬 바이너리를 시스템 위치에 복사하고 `/Library/LaunchDaemons/`에 plist를 심는다. Apple 권장 방식인 `SMAppService`는 Developer ID 코드 서명이 있어야 동작해서, 서명 인증서 없이는 개발 자체가 막힌다.
- **규칙 파일 위치**: `~/Library/Application Support/com.movie42.break/rules.json` 그대로 둔다. 초기 셋업 spec은 시스템 위치로 옮길 예정이라 적었지만, 그러면 일반 사용자로 도는 GUI가 그 디렉토리에 쓸 수 없어 권한 손질이 따라붙는다. 대신 설치할 때 이 경로를 plist의 실행 인자로 데몬에 넘긴다. 다중 사용자 계정은 지원하지 않는다.
- **Windows**: `WindowsEnforcer`는 지금처럼 `NotPrivileged`를 반환한 채 둔다. Windows 머신 없이 검증할 수 없어서, macOS 실동작을 끝낸 뒤 별도 작업으로 잡는다.

## Phase 1: hosts 파일 조작

`break-enforcer`의 macOS 구현을 `NotPrivileged` 반환에서 실제 동작으로 바꾼다. 파일을 건드리는 코드와 문자열을 만드는 코드를 나눠서, 문자열 쪽은 root 없이 테스트한다.

- [x] 관리 블록 문자열 함수 (`src/system/break-enforcer/src/hosts.rs`)
  - `render_block(sites)` — 마커로 감싼 블록 텍스트를 만든다. 형식은 `spec.md`의 "hosts 블록 형식"
  - `strip_block(content)` — 기존 관리 블록만 잘라낸다. 사용자가 직접 쓴 줄은 건드리지 않는다
  - `apply_to_content(content, sites)` — 위 둘을 합쳐 최종 파일 내용을 만든다. 순수 함수라 임시 문자열로 테스트한다
  - 패턴: `break-core/src/store.rs`가 경로 상수를 한 곳에 모아둔 방식과 같게, 마커 문자열도 이 파일에만 둔다
- [x] 경로를 받는 읽기·쓰기 (`src/system/break-enforcer/src/hosts.rs`) — `read_and_write(path, sites)`. 기본 경로는 `/etc/hosts`, 테스트는 임시 파일 경로를 넘긴다
  - 원본 파일 권한(644)과 소유자를 유지한다. 같은 디렉토리에 임시 파일을 쓰고 `rename`으로 바꾼다 — 쓰는 도중 전원이 나가도 반쪽짜리 `hosts`가 남지 않는다
- [x] `MacosEnforcer` 연결 (`src/system/break-enforcer/src/macos.rs`) — `apply_sites`/`clear_sites`가 위 함수를 호출한다. `apply_apps`/`clear_apps`는 `NotPrivileged` 그대로 둔다 (다음 작업)
  - `io::ErrorKind::PermissionDenied`를 `Error::NotPrivileged`로 옮긴다. 권한 확인을 위해 `libc` 의존을 새로 넣지 않는다 — 쓰기를 시도해 보면 알 수 있다
- [x] DNS 캐시 비우기 (`src/system/break-enforcer/src/macos.rs`) — `hosts`를 고친 뒤 `dscacheutil -flushcache`와 `killall -HUP mDNSResponder`를 실행한다. 이걸 빼면 이미 조회해 둔 주소가 캐시에 남아 몇 분간 차단이 먹지 않는다
- [x] `BREAK_DRY_RUN=1`이면 파일을 쓰지 않고 로그만 남기는 기존 동작 유지 (`src/system/break-enforcer/src/macos.rs`)
  - 재사용: `is_dry_run()`, `log_dry_run_sites()`가 `lib.rs`에 이미 있다
- 검증: `cargo test -p break-enforcer` — 빈 파일에 블록 추가, 블록 있는 파일에 다시 적용(중복 없음), 블록 제거 후 원본 줄 보존, 사용자가 쓴 줄이 살아남는지. 기존 `not_privileged.rs` 테스트는 `apply_apps` 쪽만 남기고 사이트 쪽은 새 테스트로 대체한다.

## Phase 2: 데몬 상주 루프

`break-daemon`을 버전만 출력하는 껍데기에서 실제 루프로 바꾼다.

- [x] 실행 인자 파싱 (`src/system/break-daemon/src/main.rs`) — `--rules <경로>` (필수), `--interval <초>` (기본 1), `--once` (한 번만 맞추고 종료). 인자 파서 크레이트를 새로 넣지 않고 `std::env::args`로 직접 읽는다. 인자가 세 개뿐이라 의존성값을 못 한다
- [x] 조정 루프 (`src/system/break-daemon/src/reconcile.rs`) — 매 틱마다 규칙 파일을 다시 읽고, 지금이 차단 구간인지 판정하고, `hosts`의 현재 내용과 있어야 할 내용을 비교해 다를 때만 쓴다
  - 재사용: `break_core::store::load_from`, `break_core::schedule::is_blocking_now`
  - 메모리에 "마지막으로 적용한 상태"를 들고 있지 않고 매번 파일을 읽어 비교한다. 사용자가 `hosts`를 직접 지워도 다음 틱에 되돌아온다
- [x] 규칙 파일이 없거나 읽히지 않을 때 (`src/system/break-daemon/src/reconcile.rs`) — 차단 없음으로 보고 블록을 지운다. 처리 규칙은 `spec.md`
- [x] 로그 (`src/system/break-daemon/src/main.rs`) — 상태가 바뀔 때만 한 줄씩 stdout에 남긴다. 매 틱 찍으면 1초에 한 줄씩 쌓인다
- 검증: 임시 규칙 파일을 만들고 `sudo target/debug/break-daemon --rules <임시파일> --once` 실행 → `/etc/hosts`에 블록이 생긴다. 시간대를 지나간 시각으로 바꿔 다시 실행 → 블록이 사라진다. `BREAK_DRY_RUN=1`로는 sudo 없이도 로그만 찍고 끝난다.

## Phase 3: 데몬 설치와 제거

- [x] LaunchDaemon plist 생성 (`src/system/break-app/src/daemon/plist.rs`) — 라벨 `com.movie42.break.daemon`, `ProgramArguments`에 데몬 경로와 `--rules <사용자 규칙 경로>`, `RunAtLoad` true, `KeepAlive` true, 로그는 `/Library/Logs/Break/daemon.log`
  - `KeepAlive`를 켜는 이유: 데몬이 죽은 채 `hosts`에 블록만 남으면 사용자가 영구히 차단된다
- [x] 데몬 바이너리 위치 해석 (`src/system/break-app/src/daemon/paths.rs`) — 순서대로 찾는다: ① 앱 번들 안 `Contents/MacOS/break-daemon`, ② 실행 파일과 같은 디렉토리, ③ `target/debug/break-daemon`. 개발 중(`tauri dev`)과 번들 실행 양쪽에서 같은 코드가 돌게 하려는 것
- [x] 설치 스크립트 실행 (`src/system/break-app/src/daemon/install.rs`) — 셸 명령 한 덩어리를 만들어 `osascript -e 'do shell script "..." with administrator privileges'`로 넘긴다. 하는 일: 바이너리를 `/Library/Application Support/Break/`에 복사, `root:wheel` 755로 맞춤, plist 작성, `launchctl bootout` 후 `bootstrap system`
  - 셸에 넘기는 문자열은 경로를 작은따옴표로 감싸고 이스케이프한다. 사용자 홈 경로에 공백이 들어갈 수 있다
- [x] 제거 스크립트 실행 (`src/system/break-app/src/daemon/install.rs`) — `launchctl bootout` → plist 삭제 → 바이너리 삭제. **바이너리를 지우기 전에 `break-daemon --rules <경로> --clear`로 `hosts` 블록을 먼저 지운다.** 순서가 뒤집히면 차단이 걸린 채 지울 수단이 사라진다
- [x] 상태 조회 (`src/system/break-app/src/daemon/status.rs`) — plist 파일 존재 여부와 `launchctl print system/com.movie42.break.daemon`의 종료 코드로 미설치 / 설치됨·정지 / 실행 중을 구분한다
- 검증: 앱에서 설치를 부르면 암호창이 한 번 뜨고, `sudo launchctl print system/com.movie42.break.daemon`이 프로세스를 보여준다. `/Library/Logs/Break/daemon.log`에 로그가 쌓인다. 제거를 부르면 plist가 사라지고 `/etc/hosts`에 블록이 남지 않는다.

## Phase 4: Tauri 커맨드와 상태 타입

GUI는 지금 자기 프로세스에서 `Enforcer`를 직접 호출한다. 사용자 권한이라 항상 실패하는데, 이제 차단은 데몬이 하므로 이 호출을 걷어낸다.

- [x] `AppState`에서 인프로세스 차단 호출 제거 (`src/system/break-app/src/commands.rs`) — `enforce()` 함수와 `EnforcementStatus`를 지우고 데몬 상태로 바꾼다
  - 현재 `to_state()`가 `load_rules`/`save_rules` 양쪽에서 `platform_enforcer()`를 부른다. 이 경로를 통째로 들어낸다
- [x] `DaemonStatus` 타입 정의 (`src/system/break-app/src/commands.rs`) — `notInstalled` / `installed` / `running`. 초기 셋업의 `EnforcementStatus`와 같은 `tag = "kind"` 방식을 쓴다
  - 패턴: 기존 `#[serde(rename_all = "camelCase", tag = "kind")]` 그대로
- [x] 커맨드 추가와 등록 (`src/system/break-app/src/commands.rs`, `src/system/break-app/src/lib.rs`) — `install_daemon`, `uninstall_daemon`, `daemon_status`
- [x] TypeScript 타입 맞추기 (`src/client/shared/types/app-state.ts`) — `enforcement: EnforcementStatus`를 `daemon: DaemonStatus`로 바꾼다
- [x] API 함수 추가 (`src/client/shared/api/rules.ts`) — `installDaemon`, `uninstallDaemon`, `daemonStatus`
  - 패턴: 기존 `invoke<AppState>("load_rules")` 형태 그대로
- 검증: `cargo test --workspace`, `bun run build`(tsc 포함), `bun run lint` 통과.

## Phase 5: 화면

- [x] 상단 안내 문구 교체 (`src/client/App.tsx`) — 데몬이 설치돼 있으면 안내를 감추고, 미설치면 설치 안내를 띄운다
  - 재사용: `src/client/shared/ui/Notice.tsx` (줄 배열을 받아 렌더)
  - 문구: `spec.md`의 "화면 문구"
- [x] 데몬 설치·제거 섹션 (`src/client/features/daemon/DaemonPanel.tsx`) — 현재 상태와 설치/제거 버튼. 문구는 `spec.md`의 "화면 문구"
  - 재사용: `src/client/shared/ui/Button.tsx`의 `primary`/`danger` 변형
- [x] 상태 화면에 데몬 상태 반영 (`src/client/features/status/StatusPanel.tsx`) — "실행 불가" 블록을 데몬 상태 표시로 바꾼다. 상태 목록은 `spec.md`
- [x] 설치 중 버튼 잠금 (`src/client/features/daemon/DaemonPanel.tsx`) — 암호창이 떠 있는 동안 두 번 눌리지 않게 한다
- 검증: 앱에서 사이트를 넣고 지금 시각을 포함하는 시간대를 만든 뒤 데몬을 설치한다. 브라우저에서 그 사이트가 열리지 않고, 시간대를 지우면 다시 열린다.

## Phase 6: 우회 확인과 복구 문서

- [x] DoH 우회 여부 확인 — Safari, Chrome, Firefox 각각에서 차단이 먹는지 실제로 확인하고 결과를 `진행 기록`에 남긴다. 브라우저가 운영체제 대신 자체 암호화 DNS를 쓰면 `hosts`가 무시될 수 있다. 뚫리는 브라우저가 있으면 방화벽(PF) 작업을 별도 계획으로 잡고 이번 작업은 확인까지만 한다
- [x] 수동 복구 절차 (`README.md`) — 앱이 지워졌거나 데몬이 죽어서 차단이 안 풀릴 때 터미널로 되돌리는 방법. `sudo launchctl bootout system/com.movie42.break.daemon`과 `hosts`에서 마커 사이를 지우는 절차
- [x] `README.md` 갱신 — "차단은 아직 실행되지 않습니다" 문단 교체, 데몬 설치 위치와 로그 경로, 규칙 파일 경로 표에 데몬 인자 설명 추가
- 검증: README에 적힌 복구 절차를 그대로 따라 해서 차단이 실제로 풀린다.

## 진행 기록

### 2026-08-31

- 결정: 권한 획득은 `osascript` 승격 + LaunchDaemon. `SMAppService`는 Developer ID 코드 서명이 있어야 등록되는데 인증서가 없다.
- 결정: 규칙 파일은 사용자 경로에 그대로 두고 설치 시 plist 인자로 데몬에 넘긴다. 시스템 위치로 옮기면 GUI가 쓸 수 없어 디렉토리 권한을 손대야 한다. 다중 사용자 계정은 지원하지 않는다.
- 결정: 이번 작업은 사이트 차단까지. 앱 차단은 프로세스 종료 권한과 번들 ID 탐색이 별도 난제라 다음 작업으로 뺀다.
- 결정: Windows는 다음 작업. Windows 머신 없이 서비스 설치와 hosts 갱신을 검증할 수 없다.
- 결정: 화면 문구 5종을 계획서 제안대로 확정. `spec.md`의 "화면 문구"에 옮겼다.
- Phase 1 완료
- 검증: `cargo test -p break-enforcer` — 11개 통과
- 결정: 마커 블록 앞에 빈 줄 하나를 넣는다. 기존 hosts 줄과 붙어 읽기 어려워진다.
- Phase 2 완료
- 검증: `cargo test -p break-daemon` 9개 통과. `BREAK_DRY_RUN=1 break-daemon --rules <임시파일> --once` — 차단 구간이면 "차단 적용: youtube.com", 시간대를 비우면 "차단 해제".
- 결정: `--clear`도 `--rules`를 요구한다. 인자 검사를 한 갈래로 두는 편이 단순하고, 제거 스크립트는 어차피 경로를 알고 있다.
- 블로커: `/etc/hosts`에 실제로 쓰는 확인은 sudo 암호가 필요해 세션에서 실행하지 못했다. Phase 3 설치 검증과 함께 사용자가 직접 확인한다.
- Phase 3 완료
- 검증: `cargo test -p break-app` 5개 통과 (경로 따옴표 처리, 제거 순서, AppleScript 이스케이프, 취소 판별). `launchctl print system/<label>`은 root 없이도 되는 것을 확인 — 있으면 0, 없으면 113.
- 결정: plist는 앱이 임시 파일에 먼저 쓰고 권한 승격 스크립트가 복사한다. XML을 AppleScript 문자열 안에 넣으면 이스케이프가 두 겹이 된다.
- 결정: 데몬 바이너리 후보를 두 개로 줄였다. `tauri dev`의 `target/debug/break-app`과 번들의 `Contents/MacOS/Break`는 둘 다 "실행 파일과 같은 디렉토리"로 같은 규칙에 걸린다.
- 블로커: 실제 설치·제거 확인은 관리자 암호가 필요해 사용자가 직접 실행한다.
- Phase 4 완료
- 결정: 취소를 설치와 제거에서 다르게 다룬다. 설치 취소는 "설치를 취소했습니다."를 띄우고, 제거 취소는 아무것도 바뀌지 않았으므로 화면만 다시 읽는다.
- Phase 5 완료
- 검증: `cargo test --workspace` 14개 스위트 통과, `bun run build`(tsc 포함) 통과, `bun run lint` 통과.
- 블로커: 제거 실패 문구가 `spec.md`에 없다. 지금은 데몬 계층의 오류 문자열을 그대로 띄운다 — `spec.md`의 "결정이 필요한 부분" 참고.
- 검증: `cargo check -p break-enforcer --target x86_64-pc-windows-msvc` 통과. README의 복구 절차에 적힌 마커 문자열이 `hosts.rs`의 `BEGIN_MARKER`와 글자까지 같은 것을 확인.
- 블로커: DoH 우회 확인(Safari·Chrome·Firefox)은 데몬 설치 후 브라우저를 직접 열어야 해서 남아 있다.
- 검증: `sudo break-daemon --rules <임시파일> --once` → `/etc/hosts`에 블록 생성, `dscacheutil -q host`가 `127.0.0.1`/`::1`을 반환. 브라우저에서 실제로 차단 확인.
- 결정: 이미 열려 있던 브라우저는 껐다 켜야 차단이 먹는다. 브라우저가 운영체제와 별개로 주소를 캐시하고 연결을 유지하기 때문. 한계로 기록하고 앱에서 브라우저를 끄지는 않는다.
- 검증: DoH 우회 없음. Safari, Chrome, Aside(Chromium 기반 AI 브라우저) 세 곳 모두에서 `youtube.com`이 열리지 않았다. Firefox는 이 Mac에 설치돼 있지 않아 확인하지 못했다. 방화벽(PF) 작업은 잡지 않는다.
- 결정: 제거 실패 문구는 "제거에 실패했습니다: {오류}". 설치 실패 문구와 대칭.
- Phase 6 완료
- 전체 구현 완료

### 2026-08-31 (후속)

## 변경

- `break-app/src/daemon/install.rs` — 데몬 바이너리를 임시 폴더에 먼저 복사하고 승격된 셸이 거기서 가져가도록 바꿈. 두 스크립트 모두 `cd /`로 시작.

앱에서 설치를 눌렀을 때 `cp: Operation not permitted`로 실패했다. macOS 접근 보호(TCC)는 `~/Documents` 아래를 root에게도 막는데, 프로젝트가 거기 있어서 승격된 셸이 `target/debug/break-daemon`을 읽지 못했다.
