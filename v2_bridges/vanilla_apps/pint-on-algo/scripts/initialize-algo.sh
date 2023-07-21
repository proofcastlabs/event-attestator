#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo [+] Initializing \'$BINARY_NAME\'s HOST side...

../../scripts/build-binary.sh $BINARY_NAME

../../../../target/release/$BINARY_NAME initializeAlgo \
--appId=1337 \
--genesisId="mainnet-v1.0" \
--fee=1000 \
--confs=1 \
--file=./algo-init-block.json

echo [+] \'$BINARY_NAME\'s HOST side initialized!
