#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo [+] Initializing \'$BINARY_NAME\'s NATIVE side...

../../scripts/build-binary.sh $BINARY_NAME

../../../../target/release/$BINARY_NAME initializeInt \
--confs=0 \
--chainId=3 \
--gasPrice=20000000000 \
--vaultAddress="0x7f101fe45e6649a6fb8f3f8b43ed03d353f2b90c" \
--routerAddress="0x5c904ff7853fd644fda31f8aaeaa9ec71e90ee39" \
--file=./int-init-block.json

echo [+] \'$BINARY_NAME\'s NATIVE side initialized!
