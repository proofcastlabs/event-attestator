#!/bin/bash
set -e
cd $(dirname -- $0)

. ./scripts/get-binary-name.sh

echo [+] Running all \'$(getBinaryName)\' tests...

./scripts/test-adding-debug-signer.sh
./scripts/test-core-initialization.sh
./scripts/test-native-block-submission.sh
./scripts/test-adding-multiple-debug-signers.sh
./scripts/test-native-multiple-block-submission.sh
