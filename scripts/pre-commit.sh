#!/bin/bash

# NOTE: This file is very special and intended to be separate from the rest of
# the project to minimize false-positives of pre-commit checking including files
# not going to be committed (right now, only this file).
# And so everything is inlined and no other files are referenced/used.

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
# NOTE(SAFER-BASH-AGAINST-LAX-BEHAVIOR)
set -o noclobber
set -o noglob
set -o nounset
set -o pipefail

# NOTE(SIMPLE-LOCALE-FOR-CONSISTENT-BEHAVIOR)
export LANG=C
export LANGUAGE=C
export LC_ALL=C

SLIPPAGE_BOTH_PROJECT_AND_DIR_NAME="lop"
SLIPPAGE_LOP_LAUNCHPAD_DIR="/tmp/${SLIPPAGE_BOTH_PROJECT_AND_DIR_NAME}"

function slippage-clean() {
  rm --force --recursive "${SLIPPAGE_LOP_LAUNCHPAD_DIR}"
}

function slippage-on-exit() {
  # FIXME: `$?` can incorrectly be zero/success. For example, when the script is
  # just run then followed by `Ctrl+C`.
  # This applies to other traps/files too.
  local SLIPPAGE_EXIT_CODE="$?"

  # NOTE(SOUND-NOTIFICATION)
  mpv \
    --really-quiet \
    \
    /usr/share/sounds/freedesktop/stereo/phone-incoming-call.oga \
    \
    2>/dev/null \
  || echo "can't play sound notification and that's fine." >&2

  if [[ "${SLIPPAGE_EXIT_CODE}" = "0" ]]; then
    echo "===================PRE-COMMIT CHECKS SUCCEEDED===================" >&2
  else
    echo "===================PRE-COMMIT CHECKS FAILED===================" >&2
  fi

  slippage-clean
}

trap slippage-on-exit EXIT

SLIPPAGE_LOP_SOURCE_DIR="$(pwd -P)"
if [[ \
    "$(basename "${SLIPPAGE_LOP_SOURCE_DIR}")" != \
    "${SLIPPAGE_BOTH_PROJECT_AND_DIR_NAME}" \
  ]]; then
  echo "Invalid project-directory (${SLIPPAGE_LOP_SOURCE_DIR})" >&2
  exit 1
fi

if [[ \
  "${0}" != "./scripts/pre-commit.sh" && \
  "${0}" != ".git/hooks/pre-commit"
  ]]; then
  echo "Invalid script-file (${0})" >&2
  exit 1
fi

SLIPPAGE_LOCK_FILE="/tmp/lop.lock"
SLIPPAGE_LOCK_FD="4243"
# NOTE: We hardcode `SLIPPAGE_LOCK_FD` because it only works this way.
exec 4243>|"${SLIPPAGE_LOCK_FILE}"
if ! flock --exclusive --nonblock "${SLIPPAGE_LOCK_FD}"; then
  echo "Pre-commit checks already running somewhere else!" >&2
  exit 1
fi

slippage-clean
mkdir "${SLIPPAGE_LOP_LAUNCHPAD_DIR}"
# NOTE: `/..` used in `cp` destrination as otherwise we'd have an accidentaly
# additional directory-level.
cp --recursive "${SLIPPAGE_LOP_SOURCE_DIR}" "${SLIPPAGE_LOP_LAUNCHPAD_DIR}/.."
cd "${SLIPPAGE_LOP_LAUNCHPAD_DIR}"

# NOTE(GIT-RESET-FOR-PRE-COMMIT-CHECK)
git restore .
git submodule --quiet foreach --recursive 'git restore .'
git clean -dx --force --quiet
git submodule --quiet foreach --recursive 'git clean -dx --force --quiet'
# FIXME: Check indeed no pending changes other than staging-area.

CONTAINER_COMMAND=podman \
  PRE_COMMIT_CHECK=1 \
  RUN_IN_CONTAINER=1 \
  ./scripts/continuous-integration.sh
