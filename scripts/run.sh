#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

# FIXME: Remove this file once Flutter frontend has feature-parity.
trap ./scripts/notify-user.sh EXIT
(cd ./boa && RUST_BACKTRACE=1 ./target/release/boa)
