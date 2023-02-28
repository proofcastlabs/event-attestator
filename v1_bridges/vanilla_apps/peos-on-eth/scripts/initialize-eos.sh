#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

../../scripts/build-binary.sh $BINARY_NAME

echo ✔ Initializing \'$BINARY_NAME\'s NATIVE side...

../../target/release/$BINARY_NAME initializeEos \
--chainId="aca376f206b8fc25a6ed44dbdc66547c36c6c33e3a119ffbeaef943642f0e906" \
--accountName="t11ppntoneos" \
--file=./eos-submission-material.json

echo ✔ \'$BINARY_NAME\'s NATIVE side initialized!
