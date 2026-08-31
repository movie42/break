# Break 초기 셋업

> 이슈: 없음
> 작업 종류: 기능 개발
> 상태: 완료
> 생성일: 2026-08-31

## 개요

지정한 시간대에 지정한 웹사이트와 앱을 차단하는 macOS/Windows 데스크톱 앱 Break를 만든다. 저장소는 `.git`과 이 문서만 있는 빈 상태다.

이 문서의 범위는 **차단 기능이 아니라 그 기능이 올라앉을 뼈대**다. 차단을 실제로 실행하는 코드는 한 줄도 넣지 않는다. 대신 규칙을 어떤 모양으로 저장할지, 시간 판정을 누가 할지, 차단 실행부를 어디에 둘지를 정하고 그 자리를 비워둔다.

작업 순서는 이렇게 간다:

| 순서 | 작업 | 결과물 |
| --- | --- | --- |
| 1 | **이번 셋업** | 창이 뜨고, 사이트·시간대를 편집해 규칙 파일에 저장된다. 차단은 되지 않는다 |
| 2 | 사이트 차단 | 관리자 권한 데몬이 `hosts`를 갱신한다. 여기서 앱이 처음으로 쓸모가 생긴다 |
| 3 | 앱 차단 | 같은 데몬에 프로세스 종료를 추가한다 |

2번을 먼저 하는 이유는 사이트 차단이 이 앱의 1순위 목적이기 때문이다. 그리고 사이트 차단은 사용자 권한으로 흉내조차 낼 수 없다 — `hosts` 파일이 root 소유라서, 로그인한 사용자로 뜬 GUI는 쓰기가 거부된다. 그래서 이번 셋업에서 반쯤 만들어 두는 것이 불가능하고, 2번 작업에서 데몬과 함께 통째로 만든다.

**이번 셋업의 성패는 2번에서 차단 로직을 얹을 때 뼈대를 뜯어고치지 않아도 되느냐에 달려 있다.** 그래서 차단 실행부를 Tauri에 의존하지 않는 별도 크레이트로 미리 떼어둔다. Tauri 의존이 섞이면 GUI 없이 도는 root 데몬에 웹뷰가 딸려온다.

- 포함: 프로젝트 스캐폴딩, Cargo 워크스페이스, 규칙 파일 포맷과 스케줄 판정, `trait Enforcer` 정의, 사이트·시간대를 편집하는 GUI, CI
- 제외: 사이트 차단 실행, 앱 차단 실행, 데몬, 우회 방지 장치, 모바일

## Phase 1: 프로젝트 스캐폴딩

- [x] `create-tauri-app`으로 React + TypeScript + Vite 템플릿 생성 (`package.json`, `vite.config.ts`, `src/system/break-app/`)
  - 패턴: 저장소가 비어 있어 재사용 대상 없음. 템플릿 구조를 그대로 출발점으로 삼는다
  - 패키지 매니저는 pnpm. 의존성 버전은 템플릿이 해석한 값을 쓰고 실제 버전을 `진행 기록`에 남긴다 — 이 계획을 쓴 시점에는 네트워크가 막혀 crates.io 최신 버전을 확인하지 못했다
- [x] 앱 이름 설정 (`src/system/break-app/tauri.conf.json`) — `productName: "Break"`, `identifier: "com.movie42.break"`
  - `break`는 Rust 키워드라 cargo가 패키지명으로 거부한다. 크레이트에는 접미사를 붙인다
- [x] Tailwind CSS 설치와 설정 (`tailwind.config.js`, `src/client/index.css`)
- [x] 런타임 버전 고정 (`.bun-version`) — bun 1.4.0
- [x] `.gitignore` 작성 — `node_modules/`, `dist/`, `target/`, `.DS_Store`
- [x] ESLint + Prettier 설정 (`eslint.config.js`, `.prettierrc`), `package.json`에 `lint` 스크립트 추가
- 검증: `pnpm tauri dev`로 빈 창이 뜬다. `pnpm lint`가 통과한다.

## Phase 2: Cargo 워크스페이스

- [x] 루트 `Cargo.toml`을 워크스페이스로 전환하고 멤버 4개를 등록
- [x] `src/system/break-core/` — 규칙 타입, 스케줄 판정, 규칙 파일 입출력. 플랫폼 API에도 Tauri에도 의존하지 않는다
- [x] `src/system/break-enforcer/` — 차단 실행부. 이번엔 인터페이스만 채운다. 플랫폼 API에는 의존하되 Tauri에는 의존하지 않는다
- [x] `src/system/break-daemon/` — 다음 작업에서 채울 상주 바이너리. 이번엔 `main`이 버전만 출력하는 껍데기로 둔다
  - 지금 자리를 잡아두면 다음 작업이 워크스페이스 구조를 건드리지 않고 이 크레이트 안에서만 끝난다
  - 바이너리 이름을 `break`가 아닌 `break-daemon`으로 두는 이유는 `break`가 셸 빌트인이라서다
- [x] `src/system/break-app/`가 `break-core`와 `break-enforcer`를 path 의존으로 참조
- [x] Windows 타깃 설치 — `rustup target add x86_64-pc-windows-msvc`
- 검증: `cargo check --workspace` 통과. 이어서 `cargo check -p break-enforcer --target x86_64-pc-windows-msvc`가 통과한다 — `check`는 링크를 하지 않으므로 MSVC 툴체인 없이 Windows 코드의 컴파일 여부를 Mac에서 확인할 수 있다. 툴체인 문제로 실패하면 매달리지 말고 Phase 5의 CI로 검증을 넘긴다.

## Phase 3: 규칙 파일과 스케줄 판정

이번 셋업에서 유일하게 분기가 있는 로직이고, 다음 작업의 데몬이 그대로 가져다 쓴다.

- [x] 규칙 타입 정의 (`src/system/break-core/src/rules.rs`) — `Rules`, `SiteTarget`, `AppTarget`, `Schedule`, `TimeWindow`. serde 파생 적용
  - `AppTarget`은 3번 작업에서 쓰지만 파일 포맷을 나중에 깨지 않으려면 지금 넣어둔다
- [x] 스케줄 판정 함수 (`src/system/break-core/src/schedule.rs`) — 주어진 시각이 차단 구간인지 반환. 자정을 넘는 구간의 판정 규칙은 `spec.md`에 있다
- [x] 규칙 파일 입출력 (`src/system/break-core/src/store.rs`) — JSON. 파일 경로 상수는 여기 한 곳에만 둔다. 경로가 사용자 디렉토리에서 시스템 디렉토리로 바뀌는 게 다음 작업의 첫 단계라, 흩어져 있으면 그때 전부 찾아다녀야 한다
- [x] TypeScript 타입 (`src/client/shared/types/rules.ts`) — Rust 타입과 필드명을 맞춘다. `rename_all = "camelCase"`를 걸지 정하고 양쪽에 동일하게 적용
- 검증: `cargo test -p break-core` — 직렬화 왕복, 자정 넘김 판정, 깨진 JSON 처리 테스트가 통과한다.

## Phase 4: Enforcer 인터페이스

- [x] `trait Enforcer` 정의 (`src/system/break-enforcer/src/lib.rs`) — `apply_sites`, `clear_sites`, `apply_apps`, `clear_apps`
- [x] macOS·Windows 구현체 골격 (`src/system/break-enforcer/src/macos.rs`, `windows.rs`) — 모든 메서드가 `Error::NotPrivileged`를 반환한다. `unimplemented!()`로 패닉시키지 않는다 — GUI가 이 오류를 받아 안내를 띄우는 게 이번 셋업의 정상 동작이다
- [x] 드라이런 스위치 (`src/system/break-enforcer/src/lib.rs`) — `BREAK_DRY_RUN=1`이면 실제 실행 대신 대상만 로그로 남긴다. 다음 작업부터 시스템 파일과 프로세스를 건드리므로 그 전에 자리를 만들어 둔다
- 검증: `cargo test -p break-enforcer` — 각 메서드가 `NotPrivileged`를 반환하고 패닉하지 않는다.

## Phase 5: GUI와 CI

- [x] Tauri 커맨드 등록 (`src/system/break-app/src/lib.rs`) — `load_rules`, `save_rules`
- [x] 트레이 아이콘과 메뉴 (`src/system/break-app/src/tray.rs`) — 열기 / 종료. Tauri 2는 트레이가 내장이라 플러그인을 붙이지 않는다
- [x] 창을 닫아도 프로세스가 살아있게 처리 (`src/system/break-app/src/lib.rs`) — 닫기는 창을 숨기고, 종료는 트레이 메뉴로만
- [x] 사이트 목록 편집 화면 (`src/client/features/sites/`) — 도메인 추가·삭제
- [x] 시간대 편집 화면 (`src/client/features/schedule/`) — 요일 선택과 시작·종료 시각
- [x] 상태 화면 (`src/client/features/status/`) — 현재 차단 구간인지 여부와, 차단이 아직 실행되지 않는다는 안내
- [x] GitHub Actions 워크플로 (`.github/workflows/ci.yml`) — `macos-latest`와 `windows-latest`에서 `cargo check --workspace`와 `pnpm build`
- 검증: 앱에서 사이트와 시간대를 추가하고 앱을 껐다 켜면 그대로 남아 있다. React → Tauri 커맨드 → `break-core` → 디스크까지 전 구간이 연결됐다는 뜻이고, 다음 작업의 데몬은 이 파일을 읽기만 하면 된다.

## 진행 기록

### 2026-08-31

- 결정: 작업 순서는 셋업 → 사이트 차단 → 앱 차단. 사이트 차단이 이 앱의 1순위 목적이다.
- 결정: 이번 셋업에서는 차단을 전혀 실행하지 않는다. 사이트 차단은 사용자 권한으로 부분 구현이 불가능하고(hosts가 root 소유), 앱 차단만 먼저 완성하면 우선순위가 뒤집힌다.
- 결정: 차단 실행부를 `break-enforcer`, 상주 프로세스를 `break-daemon`으로 분리하고 둘 다 Tauri에 의존시키지 않는다. 다음 작업에서 GUI 없이 도는 root 프로세스를 만들 때 웹뷰가 딸려오지 않게 하기 위해서다.
- 결정: 권한이 없을 때 `unimplemented!()`로 패닉하지 않고 `Error::NotPrivileged`를 반환한다. 이번 셋업에서는 이게 오류가 아니라 정상 경로다.
- 결정: 앱 이름은 Break(한글 브레이크). 표시명과 번들 ID는 `Break` / `com.movie42.break`, 내부 식별자는 `break-core`·`break-enforcer`·`break-daemon`·`break-app`. `break` 단독은 Rust 키워드라 cargo가 거부하고 셸 빌트인이기도 하다.
- 결정: 스타일링은 Tailwind CSS.
- 결정: 차단 미동작 안내 문구는 "아직 차단은 동작하지 않습니다. 지금은 사이트와 시간대를 저장하는 것까지만 가능합니다." (사용자 지정)
- 결정: GitHub 저장소는 movie42/break, 공개.
- 결정: 모바일은 만들지 않고 `trait Enforcer` 뒤에 자리만 열어둔다. iOS는 다른 앱을 차단하려면 Apple에서 Family Controls 엔타이틀먼트를 승인받아야 해서 일정을 예측할 수 없다.
- 결정: Windows 코드는 Mac에서 `cargo check --target`으로만 검증하고, 실제 빌드·실행은 GitHub Actions의 `windows-latest` 러너에 맡긴다. Tauri의 Windows 번들링은 WebView2와 MSI/NSIS 때문에 Windows 머신이 필요하다.

- Phase 1 완료
- 검증: `pnpm lint` 통과, `pnpm build` 통과 (Tailwind CSS 6.36 kB 산출). `pnpm tauri dev` 창 확인은 Phase 2에서 워크스페이스 재구성 후로 미룸 — 지금 하면 Rust를 두 번 빌드한다
- 결정: Tailwind는 v4라 `tailwind.config.js`가 없다. `@tailwindcss/vite` 플러그인을 `vite.config.ts`에 넣고 `src/client/index.css`에 `@import "tailwindcss"` 한 줄로 설정한다
- 결정: pnpm 11이 esbuild의 postinstall을 기본 차단한다. `pnpm-workspace.yaml`의 `allowBuilds`로 허용 — 없으면 `pnpm build`가 install 단계에서 멈춘다
- 버전: Tauri 2, React 19.2, Vite 7.3.6, TypeScript 5.8.3, Tailwind 4.3.3, ESLint 10.9.1, Node v24.16.0, pnpm 11.24.0, rustc 1.88.0
- Phase 2 완료
- 검증: `cargo check --workspace` 통과 (25.55s). `cargo check -p break-enforcer --target x86_64-pc-windows-msvc` 통과 (4.62s)
- 결정: 사용자 요청으로 패키지 매니저를 pnpm에서 bun 1.4.0으로 변경. `tauri.conf.json`의 `beforeDevCommand`/`beforeBuildCommand`를 `bun run`으로 바꾸고 락파일을 `bun.lock`으로 교체했다
- 결정: 사용자 요청으로 디렉토리 구조 변경. `src-tauri/`와 `crates/`를 없애고 `src/client`(React) + `src/system`(Rust 크레이트 4개)로 나눴다. Tauri CLI는 `src-tauri`를 하드코딩하지 않고 `tauri.conf.json`을 찾아다니므로 `tauri info`로 인식을 확인했다
- 결정: `backend`가 아니라 `system`. `break-daemon`은 GUI가 네트워크로 호출하는 서버가 아니라 hosts 파일과 프로세스를 건드리는 root 상주 프로세스라, `backend`라고 부르면 다음 작업에서 구조를 오해한다
- Phase 3 완료
- 검증: `cargo test -p break-core` 15개 통과 (직렬화 왕복·camelCase 필드명·자정 넘김 판정·경계 [시작, 종료)·겹침·깨진 JSON 격리·도메인 정규화·중복 거부)
- 결정: Rust와 TS 양쪽 필드명은 camelCase. Rust는 `#[serde(rename_all = "camelCase")]`, TS는 그대로 `bundleId`/`displayName`
- 결정: 도메인 정규화는 스킴·경로·포트·`www.`를 떼고 소문자 호스트만 남긴다. `www.x.com`과 `x.com`을 같은 항목으로 본다는 뜻이고, hosts 파일에 두 줄 다 쓰는 건 다음 작업의 enforcer가 한다
- 결정: 점이 없는 이름(`localhost`)은 거부한다. hosts 차단 대상이 아니고 오타로 들어오는 쪽이 많다
- Phase 4 완료
- 검증: `cargo test -p break-enforcer` 3개 통과. `cargo check -p break-enforcer --target x86_64-pc-windows-msvc` 통과
- 결정: `platform_enforcer()`로 플랫폼 구현체를 고르고 GUI는 `trait Enforcer`만 본다. macOS/Windows 외 타깃에도 `NotPrivileged`를 돌려주는 구현을 둬서 `cargo check`가 어디서든 통과한다
- Phase 5 완료
- 검증: `cargo test --workspace` 18개 통과. `bun run lint`·`bun run build` 통과. `bun run tauri dev`로 앱이 패닉 없이 실행되고 창 프로세스가 뜨는 것을 확인했다. 화면에서 사이트·시간대를 넣고 재실행해 남는지 확인하는 건 클릭이 필요해서 `/qa`로 넘긴다
- 결정: 도메인 정규화와 중복 제거를 `save_rules`가 저장 직전에 한 번 더 돌린다. 화면과 Rust 양쪽에 같은 규칙을 두면 다음 작업에서 한쪽만 고치게 된다
- 결정: 포트 1420을 다른 프로젝트가 점유 중이라 검증할 때만 `-c`로 1431을 썼다. 저장소 설정은 Tauri 기본값 1420 그대로다
- 블로커: 없음
- 전체 구현 완료
- 결정: Node를 쓰지 않는다. `.nvmrc`를 지우고 `.bun-version`으로 bun 1.4.0을 고정했다. PATH에서 node와 npm을 빼고 `bun run lint`·`bun run build`·`bun run tauri dev`가 전부 도는 것을 확인했다
