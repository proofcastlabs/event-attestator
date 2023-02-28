#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo ✔ Initializing \'$BINARY_NAME\'s NATIVE side...

../../scripts/build-binary.sh $BINARY_NAME

../../target/release/$BINARY_NAME initializeEth \
--confs=0 \
--chainId=3 \
--gasPrice=20000000000 \
--file=./eth-submission-material.json \
--vaultAddress="0x20abcd63afff83aff658fcc776a5fd9fc2cfc099"

echo ✔ \'$BINARY_NAME\'s NATIVE side initialized!
