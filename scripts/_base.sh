#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit

# NOTE(SAFER-BASH-AGAINST-LAX-BEHAVIOR)
# SEE: https://devdocs.io/bash/the-set-builtin#set
set -o noclobber
set -o noglob
set -o nounset
set -o pipefail

# NOTE(SIMPLE-LOCALE-FOR-CONSISTENT-BEHAVIOR)
# SEE: https://unix.stackexchange.com/a/87763.
export LANG=C
export LANGUAGE=C
export LC_ALL=C
