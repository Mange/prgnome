#!/usr/bin/env bash

if ! hash systemfd 2>/dev/null; then
  echo "You must first install systemfd." >&2
  echo "  cargo install systemfd" >&2
  exit 1
fi

export RUST_BACKTRACE=1
exec systemfd --no-pid -s http::4567 -- \
  cargo watch --clear \
    -x check \
    -x test \
    -x "run -- --bind 127.0.0.1:4567 --log-level debug"
