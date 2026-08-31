# Break

지정한 시간대에 지정한 웹사이트와 앱을 차단하는 macOS/Windows 데스크톱 앱.

> macOS에서 사이트 차단이 동작합니다. 앱 차단과 Windows 구현은 아직입니다.

차단은 `/etc/hosts`에 `127.0.0.1`을 적어 넣는 방식입니다. 이 파일은 root만 쓸 수 있어서, 로그인 사용자로 뜨는 앱이 직접 고칠 수 없습니다. 대신 root로 도는 상주 프로그램(`break-daemon`)을 한 번 설치하고, 앱은 규칙 파일만 씁니다. 설치할 때 Mac 관리자 암호를 한 번 물어봅니다.

## 구조

```
src/
  client/          React + TypeScript 프론트엔드
  system/
    break-app/     Tauri 셸 — GUI와 Rust를 잇는 커맨드, 트레이
    break-core/    규칙 타입, 스케줄 판정, 규칙 파일 입출력
    break-enforcer/hosts 파일 갱신 (앱 차단은 다음 작업)
    break-daemon/  root로 도는 상주 프로세스 — 규칙을 읽고 hosts를 맞춘다
```

`break-core`와 `break-enforcer`는 Tauri에 의존하지 않습니다. GUI 없이 도는 `break-daemon`에 웹뷰가 딸려오지 않게 하기 위해서입니다.

## 개발

```bash
bun install
bun run tauri dev     # 앱 실행
bun run lint          # ESLint
bun run build         # 프론트엔드 빌드
cargo test --workspace
```

Windows 코드는 Mac에서 컴파일 여부만 확인합니다. `check`는 링크를 하지 않아 MSVC 툴체인 없이 됩니다.

```bash
rustup target add x86_64-pc-windows-msvc
cargo check -p break-enforcer --target x86_64-pc-windows-msvc
```

bun만 씁니다. 버전은 `.bun-version`에 고정돼 있고, Node는 필요 없습니다.

## 규칙 파일

| | 경로 |
| --- | --- |
| macOS | `~/Library/Application Support/com.movie42.break/rules.json` |
| Windows | `%APPDATA%\Break\rules.json` |

깨진 JSON은 덮어쓰지 않고 `rules.json.corrupt-{타임스탬프}`로 옮긴 뒤 빈 규칙으로 시작합니다.

데몬은 이 경로를 실행 인자로 받습니다. 설치할 때 앱이 자기 계정의 경로를 plist에 박아 넣기 때문에, 계정이 바뀌면 다시 설치해야 합니다. 여러 사용자 계정은 지원하지 않습니다.

## 설치되는 것

| | 경로 |
| --- | --- |
| 데몬 바이너리 | `/Library/Application Support/Break/break-daemon` |
| LaunchDaemon plist | `/Library/LaunchDaemons/com.movie42.break.daemon.plist` |
| 로그 | `/Library/Logs/Break/daemon.log` |

데몬은 1초마다 규칙 파일을 다시 읽고, 지금이 차단 구간이면 `/etc/hosts`에 마커로 감싼 블록을 넣습니다. `hosts`를 직접 지워도 다음 틱에 되돌아옵니다.

```bash
break-daemon --rules <규칙 파일 경로> [--interval <초>] [--once] [--clear]
```

## 차단이 안 풀릴 때

앱을 지웠거나 데몬이 죽어서 차단이 남아 있으면 터미널에서 직접 되돌립니다.

```bash
sudo launchctl bootout system/com.movie42.break.daemon
sudo rm -f /Library/LaunchDaemons/com.movie42.break.daemon.plist
sudo rm -f "/Library/Application Support/Break/break-daemon"
```

그다음 `/etc/hosts`를 열어 아래 두 줄과 그 사이를 지웁니다.

```
# BEGIN Break — 이 블록은 Break가 관리합니다. 직접 고치지 마세요.
# END Break
```

```bash
sudo nano /etc/hosts
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

마지막 두 줄은 DNS 캐시를 비웁니다. 이걸 빼면 이미 조회해 둔 주소가 몇 분간 남아 사이트가 계속 안 열립니다.

## 알려진 한계

- 서브도메인은 막히지 않습니다. `youtube.com`을 넣어도 `m.youtube.com`은 열립니다. `hosts`는 와일드카드를 지원하지 않습니다.
- 브라우저가 자체 암호화 DNS(DNS-over-HTTPS)를 쓰면 `hosts`를 건너뜁니다. Safari, Chrome, Aside에서는 우회가 없는 것을 확인했습니다. Firefox는 확인하지 못했습니다.
- **이미 열려 있던 브라우저는 바로 막히지 않습니다.** 브라우저는 운영체제와 별개로 자기 안에 주소를 캐시하고 연결을 유지합니다. 차단이 걸린 뒤 브라우저를 껐다 켜야 적용됩니다.
- 우회 방지 장치가 없습니다. 데몬을 제거하거나 규칙 파일을 지우면 차단이 풀립니다.

## 문서

앞으로의 방향은 `docs/roadmap.md`에 있습니다. 지난 작업의 계획과 명세는
`docs/initial-setup/plans/`와 `docs/site-blocking/plans/`에 있습니다.
