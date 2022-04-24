#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

trap ./scripts/notify-user.sh EXIT
RUN_IN_CONTAINER=1 ./scripts/continuous-integration.sh podman
