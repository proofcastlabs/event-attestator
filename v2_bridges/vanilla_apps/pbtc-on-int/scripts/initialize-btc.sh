#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo [+] Initializing \'$BINARY_NAME\'s NATIVE side...

../../scripts/build-binary.sh $BINARY_NAME

../../../../target/release/$BINARY_NAME initializeBtc \
--fee=23 \
--confs=0 \
--difficulty=0 \
--network="Testnet" \
--file=btc-submission-material.json

echo [+] \'$BINARY_NAME\'s NATIVE side initialized!

