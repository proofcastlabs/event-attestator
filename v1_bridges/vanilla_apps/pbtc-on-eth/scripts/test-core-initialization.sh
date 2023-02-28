#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

../../scripts/clean-up.sh $BINARY_NAME

echo ✔ Testing \'$BINARY_NAME\' core initialization...

./initialize-eth.sh
./initialize-btc.sh
../../target/release/$BINARY_NAME getEnclaveState
../../scripts/clean-up.sh $BINARY_NAME

echo ✔ \'$BINARY_NAME\' core initialization test successful!
