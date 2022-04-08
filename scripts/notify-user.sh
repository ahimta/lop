#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

mpv --really-quiet /usr/share/sounds/freedesktop/stereo/phone-incoming-call.oga 2>/dev/null ||
  echo "can't play sound notification." >&2
