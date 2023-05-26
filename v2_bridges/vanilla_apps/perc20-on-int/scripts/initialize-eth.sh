#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh
BINARY_NAME=$(getBinaryName)

echo [+] Initializing \'$BINARY_NAME\'s NATIVE side...

../../scripts/build-binary.sh $BINARY_NAME

../../../../target/release/$BINARY_NAME initializeEth \
--routerAddress=0x8549cf9b30276305de31fa7533938e7ce366d12a \
--vaultAddress=0x8549cf9b30276305de31fa7533938e7ce366d12a \
--confs=0 \
--chainId=11155111 \
--gasPrice=20000000000 \
--file=./eth-init-block.json

echo [+] \'$BINARY_NAME\'s NATIVE side initialized!
