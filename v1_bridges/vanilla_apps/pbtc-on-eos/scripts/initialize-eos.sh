#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo ✔ Initializing \'$BINARY_NAME\'s HOST side...

../../scripts/build-binary.sh $BINARY_NAME

../../target/release/$BINARY_NAME initializeEos \
"pbtctokenxxx" \
--chainId="e70aaab8997e1dfce58fbfac80cbbb8fecec7b99cf982a9444273cbc64c41473" \
--symbol="PBTC" \
--file=./eos-submission-material.json

echo ✔ \'$BINARY_NAME\'s HOST side initialized!
