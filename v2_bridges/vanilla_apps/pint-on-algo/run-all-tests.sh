#!/bin/bash
set -e
cd $(dirname -- $0)

. ./scripts/get-binary-name.sh

echo running all \'$(getBinaryName)\' tests...

array=( $(find ./scripts -name "test-*.sh" -o -name "*-test.sh -print0") )

for t in "${array[@]}"
do
  echo running test script: \'$t\'...
  $t
done

