#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

# FIXME: Remove this file once Flutter frontend has feature-parity.
trap ./scripts/notify-user.sh EXIT

(
  cd ./boa
  # NOTE(RUST-BACKTRACE-FULL): This value is suggested by a release run
  # backtrace but not documented.
  # SEE: https://doc.rust-lang.org/std/backtrace/index.html#environment-variables
  RUST_BACKTRACE=full cargo run \
    --quiet \
    --jobs "$(nproc)" \
    --no-default-features
)
