#!/bin/bash

# SEE: https://devdocs.io/bash/the-set-builtin#set
# NOTE: `errexit` set first to catch errors with other `set`s.
set -o errexit

set -o noclobber
set -o noglob
set -o nounset
set -o pipefail
