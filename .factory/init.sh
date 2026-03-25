#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if [ ! -f ./.env ]; then
  cat > ./.env <<'EOF'
TOGGL_API_TOKEN=fake-local-dev-token
TOGGL_API_URL=https://opentoggl.invalid/api/v9
TOGGL_DISABLE_HTTP_CACHE=1
EOF
fi

set -a
[ -f ./.env ] && . ./.env
set +a

cargo fetch >/dev/null 2>&1 || cargo fetch
