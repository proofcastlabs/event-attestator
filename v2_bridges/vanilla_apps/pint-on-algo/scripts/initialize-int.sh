#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo [+] Initializing \'$BINARY_NAME\'s HOST side...

../../scripts/build-binary.sh $BINARY_NAME

../../../../target/release/$BINARY_NAME initializeInt \
--vaultAddress 0x88dfbcBB77C7b648F546227C2aB03b04143C37ce \
--routerAddress 0x28fDb6777285366b5aEe0D41aAa2cB3388D5CbeF \
--confs=0 \
--chainId=1 \
--gasPrice=1000000000 \
--file=./int-init-block.json

echo [+] \'$BINARY_NAME\'s HOST side initialized!
