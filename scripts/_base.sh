#!/bin/bash

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
