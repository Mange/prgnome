#!/usr/bin/env bash

if ! hash systemfd 2>/dev/null; then
  echo "You must first install systemfd." >&2
  echo "  cargo install systemfd" >&2
  exit 1
fi

export RUST_BACKTRACE=1 RUST_LOG=actix_web=debug
exec systemfd --no-pid -s http::4567 -- cargo watch --clear -x check -x test -x run
