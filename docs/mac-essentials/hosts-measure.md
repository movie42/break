# hosts 8만 줄 속도 측정

상태: 🟡 아직 재지 않음 — `/etc/hosts`를 고쳐야 해서 관리자 암호가 필요하다. 그때까지는 hosts 방식으로 넣어 둔다.

## 재는 법

```sh
sh scripts/fetch-adult-list.sh
sudo sh scripts/measure-hosts.sh
```

스크립트는 지금 hosts를 백업하고, 유해사이트 목록을 한 줄씩(`0.0.0.0 주소`) 붙인 뒤, 막히지 않은
사이트 다섯 곳의 주소 찾기 시간(`getaddrinfo`)을 20번씩 재고, 끝나면 hosts를 원래대로 되돌린다.
넣기 전과 넣은 뒤의 평균·p95·최대값이 나온다.

## 판단 기준 (PRD F5)

| 결과 | 다음 |
| --- | --- |
| 평균 20ms 이하로 늘고, 1초 넘게 멈춘 적이 없다 | hosts를 유지한다 |
| 평균 20ms 넘게 늘거나, 한 번이라도 1초 넘게 멈췄다 | 로컬 DNS로 간다. 별도 계획을 쓴다 |

## 목록

- 출처: StevenBlack/hosts `alternates/porn-only`, `alternates/gambling-only` (MIT, 원본 일부 CC BY-SA)
- 2026-10-18에 받았고 주소 83,510개, 1.4MB다 (`src/system/break-app/resources/adult-hosts.txt`)
- hosts에는 주소 하나당 한 줄만 쓴다. 직접 넣은 사이트처럼 IPv4·IPv6 × www 네 줄로 쓰면 33만 줄이 된다

## 결과

(측정 후 여기에 적는다)
