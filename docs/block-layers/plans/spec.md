# 차단 계층 정리 - 명세

## 화면/기능 흐름

- 차단 구간 진입: 데몬이 hosts 블록을 쓰고, PF 앵커를 올리고, 정책 plist를 쓴다 → 정책 파일이
  이번 tick에 실제로 바뀌었으면 설정 캐시를 비운다
- 차단 구간 이탈: hosts 블록 제거 + PF 앵커 비우기 + 정책 plist 삭제 → **파일이 실제로 지워졌으면
  설정 캐시를 비운다.** 이걸 안 하면 브라우저는 지워진 정책을 계속 본다
- 같은 내용으로 다음 tick: 파일이 그대로이므로 아무것도 쓰지 않고 캐시도 안 비운다
- 남이 만든 정책 파일 발견: 읽고 지나간다. 쓰지도 지우지도 않고, 캐시도 안 비운다
- 데몬 제거 (`--clear`): 위 "차단 구간 이탈"과 같은 경로를 탄다
- 계측 모드 (`BREAK_SKIP_LAYERS`): 지정한 계층의 apply만 건너뛴다. clear는 항상 전부 돈다

## 상태 정의

| 상태 | 설명 |
| --- | --- |
| 정책 적용 중 | `/Library/Managed Preferences/<번들ID>.plist`가 있고, `CFPreferencesAppValueIsForced("URLBlocklist")`가 참 |
| 정책 해제됨 | 파일이 없고 `forced`가 거짓. **파일만 없는 상태는 해제가 아니다** — 이번 버그가 정확히 그 상태였다 |
| 정책 없음 | 애초에 쓴 적이 없음. 차단 구간이 아니거나 막을 사이트가 없다 |
| 남의 정책 | 파일은 있는데 `BreakManaged` 키가 없다. 우리 것이 아니므로 건드리지 않는다 |

## 판정 방법

파일 유무로 판정하지 않는다. 브라우저가 정책을 읽는 것과 같은 경로로 물어본다.

```
CFPreferencesCopyAppValue("URLBlocklist", "com.google.Chrome")
CFPreferencesAppValueIsForced("URLBlocklist", "com.google.Chrome")
```

`defaults read com.google.Chrome URLBlocklist`는 쓰지 않는다. 2026-09-03에 정책이 살아 있는
상태에서 `does not exist`를 돌려줬다. 관리 설정 계층을 안 보는 것으로 보인다.

## 실측 절차

launchd에 등록된 데몬은 환경변수를 받지 않는다. 계측할 때는 그 데몬을 내리고, 같은 바이너리를
터미널에서 직접 띄운다. 계측용 스위치가 shipping plist에 새지 않는다.

### 준비 (한 번만)

1. 앱에서 그룹 하나에 `youtube.com`만 담고, 지금 시각을 포함하는 시간대를 켠다.
   규칙 파일은 `~/Library/Application Support/com.movie42.break/rules.json`
2. 데몬 바이너리를 빌드한다 — `cargo build -p break-daemon` → `target/debug/break-daemon`
3. 설치된 데몬이 돌고 있으면 내린다 — `sudo launchctl bootout system/com.movie42.break.daemon`

편의상 아래에서 쓰는 이름:

```
RULES=~/Library/Application\ Support/com.movie42.break/rules.json
DAEMON=./target/debug/break-daemon
```

### 한 칸을 재는 순서

1. 직전 차단이 남아 있지 않은지 확인한다
   `sudo $DAEMON --rules $RULES --clear && python3 scripts/policy-probe.py --expect-clear`
2. 잴 계층 조합으로 데몬을 띄운다 (터미널 하나를 계속 잡는다)
   - 실측 A (hosts만): `sudo env BREAK_SKIP_LAYERS=policy,dns $DAEMON --rules $RULES`
   - 실측 B (hosts + PF): `sudo env BREAK_SKIP_LAYERS=policy $DAEMON --rules $RULES`
   - 실측 C (세 겹 전부): `sudo $DAEMON --rules $RULES`
3. 브라우저를 완전히 종료했다 다시 켜고 `youtube.com`을 연다 → "껐다 켬 O" 칸
4. 브라우저를 켠 채로 데몬을 띄운 경우도 한 번 잰다 → "껐다 켬 X" 칸
5. 열렸으면 브라우저 캐시를 비우고 한 번 더 재서 우연이 아닌지 본다
6. 데몬 터미널에서 Ctrl+C로 멈추고, 1번의 해제·확인을 다시 돌린다

`--clear`는 `BREAK_SKIP_LAYERS`를 보지 않는다. 계측 중에도 해제는 항상 세 겹 전부 돈다.

### 해제가 브라우저까지 닿았는지 확인

```
python3 scripts/policy-probe.py
```

브라우저마다 `forced=False value=None file=없음`이 나와야 한다. 파일이 없는데 `forced=True`이면
설정 캐시가 아직 옛 값을 들고 있다는 뜻이고, 그게 이번에 고친 버그다.

## 실측 결과

조건: 막을 사이트 `youtube.com` 하나, 차단 구간 안에서 `youtube.com` 접속.
"열렸는가"가 O면 그 계층 조합으로는 못 막는다는 뜻이다. 빈 칸은 결론에 영향이 없어 재지 않았다.

| 브라우저 | 계층 | 브라우저 껐다 켬 | 열렸는가 |
| --- | --- | --- | --- |
| Chrome | hosts만 | O | |
| Chrome | hosts만 | X | |
| Chrome | hosts + PF | O | X |
| Chrome | hosts + PF | X | |
| Chrome | 세 겹 전부 | O | X |
| Aside | hosts만 | O | |
| Aside | hosts만 | X | |
| Aside | hosts + PF | O | **O** (강력 새로고침) |
| Aside | hosts + PF | X | |
| Aside | 세 겹 전부 | O | X ("조직에서 차단") |
| Safari | hosts만 | O | |
| Safari | hosts + PF | O | X |

실측 B에서 데몬 로그는 `차단 적용: youtube.com (브라우저 정책 + 보안 DNS 차단)` — PF 가드는
정상 적용된 상태였다.

Aside는 첫 접속에서 서비스워커가 들고 있던 캐시 껍데기("오프라인 상태입니다")를 보여주지만,
강력 새로고침하면 실제로 열린다. 차단 중 Aside 프로세스가 `tv-in-f95.1e100.net:443`(YouTube
실서버)로 TCP 연결을 유지하고 있었다 — hosts를 우회해 진짜 IP를 얻었다는 뜻이다.

Chrome과 Aside 모두 `dns_over_https` 설정이 비어 있다(둘 다 Chromium 기본값). 같은 기본값에서
Chrome은 막히고 Aside만 뚫리는 이유는 아직 모른다.

실측 C(세 겹 전부)에서 Aside는 "조직에서 차단했습니다" 화면을 냈다. hosts를 우회하더라도
`URLBlocklist` 정책은 브라우저 안에서 URL 자체를 막기 때문에 DNS가 어디로 가든 상관이 없다.

## 결론 — 브라우저 정책 계층을 남긴다

실측 B에서 Aside가 뚫렸고 실측 C에서 막혔다. 세 겹 중 Aside를 막는 것은 브라우저 정책뿐이다.
hosts와 PF는 DNS 해석을 막는 방식이라, 브라우저가 자체 경로로 IP를 얻으면 둘 다 지나간다.
정책은 해석 결과와 무관하게 브라우저 안에서 URL을 막으므로 그 경로를 덮는다.

SelfControl이 hosts + PF만 쓰는 것과 다른 선택인데, SelfControl이 만들어질 때는 브라우저가
자체 DNS를 들고 다니지 않았다. Aside 같은 최근 브라우저에는 그 전제가 깨진다.

해제 쪽 위험은 Phase 1에서 없앴다. 정책 파일이 실제로 바뀐 tick에서만 `killall cfprefsd`를
불러 설정 캐시를 비운다. 실측 C 뒤 해제에서 "조직에서 차단" 화면이 즉시 풀렸고,
`policy-probe.py --expect-clear`가 통과했다.

## 엣지 케이스

- **`killall cfprefsd` 실패**: 무시하고 진행한다. hosts와 PF 차단은 이미 걸렸거나 풀렸고, 캐시
  플러시는 보강이다. 여기서 `Err`를 올리면 성공한 해제가 실패로 보고된다
- **캐시를 비우는 순간 다른 앱의 설정 읽기**: `cfprefsd`는 launchd가 즉시 다시 띄운다. 읽는 쪽은
  잠깐 기다렸다 받는다. 파일은 그대로이므로 값이 사라지지 않는다
- **정책 파일이 여러 개 (Chrome, Aside, Arc, ...)**: 하나라도 바뀌면 캐시를 한 번만 비운다.
  `cfprefsd`는 도메인별로 죽일 수 없다
- **차단 구간 안에서 사이트 목록만 바뀜**: 정책 내용이 달라지므로 파일을 다시 쓰고 캐시도 비운다
- **브라우저를 켠 채로 해제**: 캐시를 비워도 이미 뜬 브라우저가 정책을 언제 다시 읽는지는 브라우저
  몫이다. Phase 2의 "껐다 켬 X" 줄이 이걸 잰다. 즉시 안 풀리면 해제 시점에도 브라우저 종료를
  안내할지 따로 판단한다
- **`BREAK_SKIP_LAYERS`가 켜진 채로 데몬이 남음**: 계측 스위치는 Phase 3에서 지운다. 남겨 두면
  차단이 조용히 약해지는 경로가 된다
- **실측 중 사이트가 열림**: 그 자체가 결과다. 브라우저 캐시를 지우고 한 번 더 재서 우연이
  아닌지 확인한다
