#!/bin/bash
set -e
cd "$(dirname -- $0)"

# NOTE: This script just gives an easy hook for CI to easily run any shell tests defined in this repo.

array=( $(find ./ -name "test-*.sh" -o -name "*-test.sh -print0") )

for t in "${array[@]}"
do
  echo Running test script: \'$t\'...
  $t
done

