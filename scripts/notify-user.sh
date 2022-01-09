#!/bin/bash

set -o errexit
set -o pipefail
set -o nounset

mpv --really-quiet /usr/share/sounds/freedesktop/stereo/phone-incoming-call.oga 2>/dev/null ||
  echo "can't play sound notification." >&2
