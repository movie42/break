# Break

지정한 시간대에 지정한 웹사이트와 앱을 차단하는 macOS/Windows 데스크톱 앱.

> 현재는 초기 셋업 단계입니다. 사이트와 시간대를 편집해 규칙 파일에 저장하는 것까지 동작하고, **차단은 아직 실행되지 않습니다.**

## 구조

```
src/
  client/          React + TypeScript 프론트엔드
  system/
    break-app/     Tauri 셸 — GUI와 Rust를 잇는 커맨드, 트레이
    break-core/    규칙 타입, 스케줄 판정, 규칙 파일 입출력
    break-enforcer/차단 실행부 인터페이스 (구현은 다음 작업)
    break-daemon/  관리자 권한 상주 프로세스 (다음 작업)
```

`break-core`와 `break-enforcer`는 Tauri에 의존하지 않습니다. 다음 작업에서 GUI 없이 도는 관리자 권한 프로세스를 만들 때 웹뷰가 딸려오지 않게 하기 위해서입니다.

## 개발

```bash
bun install
bun run tauri dev     # 앱 실행
bun run lint          # ESLint
bun run build         # 프론트엔드 빌드
cargo test --workspace
```

Node는 `.nvmrc`의 v24.16.0, 패키지 매니저는 bun입니다.

## 규칙 파일

| | 경로 |
| --- | --- |
| macOS | `~/Library/Application Support/com.movie42.break/rules.json` |
| Windows | `%APPDATA%\Break\rules.json` |

깨진 JSON은 덮어쓰지 않고 `rules.json.corrupt-{타임스탬프}`로 옮긴 뒤 빈 규칙으로 시작합니다.

## 문서

계획과 명세는 `docs/initial-setup/plans/`에 있습니다.
