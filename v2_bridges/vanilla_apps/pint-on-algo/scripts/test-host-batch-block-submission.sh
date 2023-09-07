#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../../../target/release/$BINARY_NAME"

echo [+] Testing HOST batch block submission to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

getCoreLatestHostBlockNumber() {
	$BINARY_PATH getEnclaveState | jq .algo.algo_latest_block_number
}

getExpectedBlockNumber() {
	cat ./algo-subsequent-block-3.json | jq .block.round
}

./initialize-int.sh
./initialize-algo.sh

if [[ $(getCoreLatestHostBlockNumber) == null ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Something went wrong with core initalization!
	exit 1
fi

# Let's submit our sample blocks...
$BINARY_PATH submitAlgoBlocks --file=./algo-batch-of-blocks.json

[[ $(getCoreLatestHostBlockNumber) == $(getExpectedBlockNumber) ]] && result=true || result=false

../../scripts/clean-up.sh $BINARY_NAME

if [[ $result == true ]]; then
	echo [+] HOST batch block submission test to \'$BINARY_NAME\' core test PASSED!
else
	echo [-] HOST batch block submission test to \'$BINARY_NAME\' core test FAILED!
	exit 1
fi
