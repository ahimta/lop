#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

# NOTE(SOUND-NOTIFICATION)
mpv \
    --really-quiet \
    \
    /usr/share/sounds/freedesktop/stereo/phone-incoming-call.oga \
    \
    2>/dev/null \
  || echo "can't play sound notification and that's fine." >&2
