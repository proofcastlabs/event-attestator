#!/bin/bash
set -e
cd $(dirname -- $0)

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo ✔ Initializing \'$BINARY_NAME\'s HOST side...

../../scripts/build-binary.sh $BINARY_NAME

../../../../target/release/$BINARY_NAME initializeInt \
--routerAddress=0x6378FDbABc02734680D02d2414E5617772b6d475 \
--confs=0 \
--chainId=1 \
--gasPrice=20000000000 \
--file=./int-init-block.json

echo ✔ \'$BINARY_NAME\'s HOST side initialized!
