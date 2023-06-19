#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../../../target/release/$BINARY_NAME"

echo [+] Testing HOST block submission to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

getCoreLatestBlockNumber() {
	$BINARY_PATH getEnclaveState | jq .int.eth_latest_block_number
}

getInitBlockNumber() {
	cat ./int-init-block.json | jq .block.number
}

getExpectedBlockNumber() {
	cat ./int-subsequent-block-3.json | jq .block.number
}

./initialize-int.sh
./initialize-btc.sh

if [[ $(getCoreLatestBlockNumber) == null ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Something went wrong with core initalization!
	exit 1
fi

# Let's submit our sample blocks...
$BINARY_PATH submitIntBlock --file=./int-subsequent-block-1.json
$BINARY_PATH submitIntBlock --file=./int-subsequent-block-2.json
$BINARY_PATH submitIntBlock --file=./int-subsequent-block-3.json

[[ $(getCoreLatestBlockNumber) == $(getExpectedBlockNumber) ]] && result=true || result=false

../../scripts/clean-up.sh $BINARY_NAME

if [[ $result == true ]]; then
	echo [+] HOST block submission test to \'$BINARY_NAME\' core test PASSED!
else
	echo [-] HOST block submission test to \'$BINARY_NAME\' core test FAILED!
	exit 1
fi
