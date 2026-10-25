#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
  echo "관리자 권한이 필요합니다: sudo $0" >&2
  exit 1
fi

LIST="$(cd "$(dirname "$0")/.." && pwd)/src/system/break-app/resources/adult-hosts.txt"
HOSTS=/etc/hosts
BACKUP="$(mktemp)"
SAMPLES="www.apple.com www.wikipedia.org github.com www.naver.com news.ycombinator.com"
ROUNDS=20

cp "$HOSTS" "$BACKUP"
trap 'cp "$BACKUP" "$HOSTS"; rm -f "$BACKUP"; dscacheutil -flushcache; killall -HUP mDNSResponder' EXIT

measure() {
  python3 -I - "$ROUNDS" $SAMPLES <<'PY'
import socket, subprocess, sys, time
rounds = int(sys.argv[1]); names = sys.argv[2:]
times = []
for _ in range(rounds):
    for name in names:
        subprocess.run(["dscacheutil", "-flushcache"], check=False)
        start = time.perf_counter()
        try:
            socket.getaddrinfo(name, 443)
        except OSError:
            pass
        times.append((time.perf_counter() - start) * 1000)
times.sort()
print(f"avg {sum(times)/len(times):.1f}ms  p95 {times[int(len(times)*0.95)]:.1f}ms  max {times[-1]:.1f}ms")
PY
}

echo "before: $(measure)"
grep -v '^#' "$LIST" | sed 's/^/0.0.0.0 /' >> "$HOSTS"
dscacheutil -flushcache; killall -HUP mDNSResponder; sleep 2
echo "after ($(grep -vc '^#' "$LIST") lines): $(measure)"
