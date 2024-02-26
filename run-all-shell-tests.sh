#!/bin/bash

# NOTE: Exit early if non zero exit code is encountered
set -e
cd "$(dirname -- $0)"

# NOTE: This script just gives an easy hook for CI to easily run any shell tests defined in this repo.

array=( $(find ./ -name "test-*.sh" -o -name "*-test.sh -print0" | grep -v v1_bridges) )

for t in "${array[@]}"
do
  echo [+] running test script: \'$t\'...
  $t
done

