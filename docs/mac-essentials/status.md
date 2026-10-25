# 구현 상태

PRD: `docs/mac-essentials/prd.md` · 기준일: 2026-10-18 · Windows 제외 · 브랜치 `redesign`, 커밋 전

구현하면서 정한 것: `docs/mac-essentials/decisions.md`

| 기능 | 상태 | 남은 것 |
| --- | --- | --- |
| F1 즉시 시작하기 | ✅ | 실제 앱에서 1분 차단이 풀리는지 |
| F2 실시간 표시 | ✅ | — |
| F3 실행 중 연장 | ✅ | — |
| F4 추천 그룹 | ✅ | — |
| F5 성인·도박 기본 차단 | 🟡 | hosts 속도 측정, 실제 Mac에서 차단 확인 |
| F6 규칙을 차단 프로그램만 쓰는 자리로 | 🟡 | 다시 설치 후 실제 Mac에서 확인 |
| F7 조작 이력과 메일 알림 | 🟡 | 실제 메일 계정으로 발송 확인 |

## PRD 완료 기준별 확인

확인 방법: **테스트** = `cargo test`, **화면** = 브라우저에 가짜 응답을 넣은 임시 페이지, **프로세스** = 차단 프로그램을 임시 폴더·dry-run으로 띄워 소켓으로 요청, **미확인** = 관리자 암호나 메일 계정이 필요해 아직 못 함

| 기준 | 확인 |
| --- | --- |
| F1 25분 뒤 정확히 풀린다 | 테스트 (끝 시각 직전 막힘, 그 시각에 풀림) |
| F1 자정을 넘긴 즉시 차단이 다음 주에 다시 돌지 않는다 | 테스트 |
| F1 앱을 끈 채로 끝 시각이 지나도 풀린다 | 테스트 (판정은 차단 프로그램이 끝 시각만 봄) |
| F1 실행 중인 즉시 차단을 끄거나 지울 방법이 화면에 없다 | 화면 |
| F2 남은 시간이 1초마다 줄어든다 | 화면 |
| F2 점이 눈금과 모양으로 구분된다 | 화면 |
| F2 1초 갱신 범위가 배너·타임라인뿐이다 | 코드 구조 (1초 타이머가 두 컴포넌트 안에만 있음). React DevTools로는 안 봄 |
| F3 연장하면 배너의 풀리는 시각이 바뀐다 | 화면 (즉시 차단 연장 시트) |
| F3 연장으로 끝 시각을 앞당길 수 없다 | 테스트 |
| F4 버튼 한 번으로 주소가 든 그룹이 생긴다 | 타입 검사. 화면에서 눌러 보지는 않음 |
| F4 추천 그룹에서 주소를 빼고 더할 수 있다 | 일반 그룹과 같은 코드 |
| F4 주소 목록 문서 | `presets.md` |
| F5 측정 결과와 결정이 문서에 있다 | **미확인** (`hosts-measure.md`) |
| F5 동의하지 않으면 앱을 쓸 수 없다 | 화면 |
| F5 동의하면 시간대가 없어도 막힌다 | 테스트 + 프로세스 (83,510개 적용) |
| F5 끄기 신청 후 대기 동안 막히고 취소할 수 있다 | 테스트 + 프로세스 (테스트 대기 1분) + 화면 (카운트다운) |
| F5 규칙 파일을 고쳐도 대기를 건너뛸 수 없다 | 테스트 (저장 요청으로 필터를 끄면 거절) |
| F5 크롬 정책에 목록이 안 들어간다 | 테스트 + 프로세스 |
| F6 일반 권한으로 규칙 파일을 고칠 수 없다 | **미확인** (설치 후 파일 권한 0644 root) |
| F6 앱의 모든 저장이 차단 프로그램을 거친다 | 코드 구조 |
| F6 실행 중인 시간대 끄기 요청을 거절한다 | 테스트 + 프로세스 |
| F6 Break 앱이 아닌 프로그램의 요청을 거절한다 | 테스트 + 프로세스 (Python의 요청이 거절되고 기록됨) |
| F7 내렸다 올리면 꺼져 있던 구간이 뜬다 | 프로세스 (22초 내렸다 올림) + 화면 (기록 탭) |
| F7 앱이 켜진 채 차단 프로그램을 내리면 메일 | **미확인** |
| F7 차단 중 앱을 종료하면 메일 | **미확인** |
| F7 둘 다 끄고 재부팅하면 다시 켜진 뒤 메일 | **미확인** |
| F7 다른 사람 주소에 확인 메일 | **미확인** |
| F7 주소를 지우면 대기 동안 알림이 가고 예고 메일 | 테스트 (대기 후 빠짐). 메일은 **미확인** |

## 변경

- `break-core`: 규칙 v4(`instant`, `adultFilter`, `alerts`, `lock`, 그룹 `preset`), 저장 검사(`change.rs`), 비밀번호(`password.rs`), 요청 형식(`protocol.rs`), 메일(`mail.rs`, curl로 보냄), 알림 문구(`notice.rs`), 기록·생존 표시(`events.rs`)
- `break-daemon`: `/var/run/com.movie42.break.sock`로 요청을 받아 검사 후 `/Library/Application Support/Break/rules.json`에 씀. 등록된 앱(`--client`)의 요청만 받음(`peer.rs`). 사용자 폴더 규칙은 처음 켤 때 한 번 옮김. 5초마다 생존 표시, 다시 켜질 때 빈 구간을 기록·메일. 대기가 지난 신청을 반영. 유해사이트 목록은 hosts에만, 주소 하나당 한 줄
- `break-enforcer`: `apply_sites`가 hosts 변경 여부를 돌려주고, 목록용 한 줄 형식을 받음
- `break-app`: 저장은 차단 프로그램에 부탁(설치 전·이전 판 차단 프로그램일 때는 사용자 폴더에 씀). 5초마다 차단 프로그램을 확인해 꺼지면 메일, 차단 중 앱 종료 시 메일. 설치할 때 목록 복사, 제거할 때 규칙을 사용자 폴더로 되돌림
- 화면: 지금부터 막기 시트, 1초 갱신 배너·타임라인 점, 연장 버튼, 추천 그룹, 첫 실행 동의·비밀번호, 보호 탭, 기록 탭, 꺼져 있던 구간 경고
- 스크립트: `scripts/fetch-adult-list.sh`(목록 받기), `scripts/measure-hosts.sh`(속도 측정, sudo)
- 목록: `src/system/break-app/resources/adult-hosts.txt` — StevenBlack porn-only + gambling-only, 2026-10-18, 83,510개

## 진행 중 고친 것

- **설치 실패:** 설치 스크립트가 목록을 `~/Documents` 안에서 바로 복사했다. macOS는 관리자 권한 스크립트라도 Documents 폴더 접근을 막아 설치 전체가 실패했다. 목록도 임시 폴더를 거쳐 복사하게 고침
- **`www.` 주소 누락:** 목록을 읽을 때 `www.`를 떼고 중복을 지워 69,446개만 막혔다. 적힌 주소 그대로 쓰게 고침
- **hosts 크기:** 직접 넣은 사이트처럼 주소 하나당 네 줄로 쓰면 33만 줄이 된다. 목록은 한 줄로 씀
- **이전 판 차단 프로그램과의 교착:** 요청을 못 받는 옛 차단 프로그램이 깔려 있으면 동의 화면에서 저장이 막히고 설치 버튼도 가려졌다. 규칙이 아직 관리자 자리로 안 옮겨졌으면 사용자 폴더에 쓰게 고침

## 주의

- 개발 빌드(`tauri dev`)로 설치하면 끄기·주소 빼기 대기가 1분이고, 지금부터 막기에 1분 버튼이 생긴다. 정식 빌드는 24시간
- 메일 계정을 바꾸는 것은 비밀번호만 묻고 대기가 없다. 엉뚱한 계정으로 바꿔 알림을 끊을 수 있다
- 앱이 바로 보내는 메일을 위해 메일 계정을 사용자 폴더(`~/Library/Application Support/com.movie42.break/mail.json`, 0600)에도 둔다
- 크롬 정책 `URLBlocklist`는 문서상 1,000개까지만 읽는다. 실제로 재지는 않았다
- 화면 문구는 PRD 초안이다. 동의·끄기 화면 문구는 확정이 필요하다
- `cargo fmt`가 기능과 무관한 6개 파일(browsers.rs, tray.rs, policy.rs, store_io.rs, pf_conf.rs, pf_rules.rs)의 줄바꿈도 바꿨다

## 남은 일 (사용자 Mac에서)

1. 앱에서 "다시 설치" → 관리자 암호
2. 지금부터 막기 → 1분 → 막혔다가 풀리는지
3. `sudo sh scripts/measure-hosts.sh` → 결과를 `hosts-measure.md`에
4. 보호 탭에서 Gmail 앱 비밀번호 연결 → 알림 주소 추가 → 확인 메일 수신

## 다른 세션에서 이어서 하기

먼저 읽을 것: `prd.md`(무엇을), `decisions.md`(어떻게 정했는지), 이 문서(어디까지 했는지). 방향의 배경은 `docs/roadmap.md`의 "정한 것"에 있다.

**작업 트리 상태.** 브랜치 `redesign`, 전부 커밋 전이다. 이 기능과 무관한 변경 두 개가 섞여 있다: `package.json`의 `tauri:dev` 스크립트(이 작업 전부터 있었음), `cargo fmt`가 바꾼 6개 파일의 줄바꿈.

**확인 명령.** 바꾼 뒤에는 이 넷이 다 통과해야 한다.

```sh
cargo test --workspace
cargo clippy --workspace
./node_modules/.bin/tsc --noEmit -p .
./node_modules/.bin/eslint src/client
```

**앱 띄우기.** `bun run tauri:dev`. Rust를 고치면 앱은 다시 빌드되지만 차단 프로그램은 아니다. 차단 프로그램을 고쳤으면 `cargo build -p break-daemon` 후 앱에서 "다시 설치"를 눌러야 반영된다.

**관리자 암호 없이 차단 프로그램 확인하기.** 임시 폴더 경로와 dry-run으로 띄우면 hosts를 건드리지 않고 소켓 요청·기록·대기를 볼 수 있다. 소켓 경로는 100자 안쪽이어야 해서 그 폴더로 들어가 상대 경로를 쓴다.

```sh
cd <임시 폴더>   # rules.json(v4)을 미리 넣어 둔다
BREAK_DRY_RUN=1 <저장소>/target/debug/break-daemon --rules rules.json --socket sock \
  --events events.jsonl --heartbeat heartbeat --mail-account mail.json \
  --adult-list <저장소>/src/system/break-app/resources/adult-hosts.txt \
  --release-delay 60 --client <요청을 보낼 프로그램의 실행 파일 경로>
```

요청은 한 줄 JSON이다(예: `{"op":"ping"}`, `{"op":"requestAdultFilterOff","password":"..."}`). 형식은 `src/system/break-core/src/protocol.rs`의 `Request`.

**화면만 보기.** 브라우저에는 Tauri가 없어서 그대로는 "규칙을 읽는 중"에서 멈춘다. 저장소 루트에 임시 HTML을 두고 `window.__TAURI_INTERNALS__.invoke`를 가짜로 정의한 뒤 `/src/client/main.tsx`를 불러오면 vite(1420)에서 볼 수 있다. 확인이 끝나면 그 파일은 지운다.

**어디에 무엇이 있나.**

| 하는 일 | 파일 |
| --- | --- |
| 규칙 모양 | `break-core/src/rules.rs`, `src/client/shared/types/rules.ts` |
| 저장 검사·대기 | `break-core/src/change.rs` |
| 요청 처리 | `break-core/src/protocol.rs` (판단), `break-daemon/src/server.rs` (소켓) |
| 요청 보낸 프로그램 확인 | `break-daemon/src/peer.rs` |
| 차단 적용 | `break-daemon/src/reconcile.rs`, `break-enforcer/src/hosts.rs` |
| 기록·생존 표시 | `break-core/src/events.rs`, `break-daemon/src/main.rs` |
| 메일 | `break-core/src/mail.rs`, `break-core/src/notice.rs`, `break-app/src/watch.rs` |
| 설치·제거 | `break-app/src/daemon/install.rs`, `plist.rs`, `paths.rs` |
| 앱 명령 | `break-app/src/commands.rs`, `src/client/shared/api/rules.ts` |
| 화면 | `src/client/features/{schedule,groups,history,protect}` |

**다음 순서.** 위 "남은 일" 1~4를 사용자와 함께 하고, 결과를 이 문서의 기준별 표와 `hosts-measure.md`에 적는다. 측정이 기준을 넘으면 로컬 DNS 계획을 `docs/<이름>/plans/`에 새로 쓴다. 문구 확정과 "하지 않은 것"(`decisions.md`)은 그 다음이다.
