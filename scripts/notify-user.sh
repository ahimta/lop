#!/bin/bash

set -o errexit
set -o pipefail
set -o nounset

mpv --really-quiet /usr/share/sounds/freedesktop/stereo/phone-incoming-call.oga ||
  echo "can't play sound notification because mpv isn't installed" >&2
