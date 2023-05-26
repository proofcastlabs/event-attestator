#!/bin/bash

# NOTE: Exit early if any command in here fails
set -e

cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BLOCKS_PATH="./blocks.json"
BINARY_PATH="../../../../target/release/$BINARY_NAME"

# NOTE: Keep track of the last executed command...
trap 'last_command=$current_command; current_command=$BASH_COMMAND' DEBUG

function maybeCleanUpSubmissinFile() {
	if [[ -f $BLOCKS_PATH ]]; then
		rm $BLOCKS_PATH > /dev/null
	fi
}

function cleanUpAndExit() {
	../../scripts/clean-up.sh $BINARY_NAME
	maybeCleanUpSubmissinFile

	if [[ $? == 0 ]]; then
		echo [+] HOST multiple block submission test to \'$BINARY_NAME\' core test PASSED!
	else
		echo [-] \'${last_command}\' command failed with exit code $?.
		echo [-] HOST block submission test to \'$BINARY_NAME\' core test FAILED!
		exit 1
	fi
}

trap cleanUpAndExit EXIT

echo [+] Testing multiple HOST block submission to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME
maybeCleanUpSubmissinFile

getCoreLatestHostBlockNumber() {
	$BINARY_PATH getEnclaveState | jq .int.eth_latest_block_number
}

getExpectedBlockNumber() {
	cat ./int-subsequent-block-3.json | jq .block.number
}

./initialize-int.sh
./initialize-evm.sh

if [[ $(getCoreLatestHostBlockNumber) == null ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Something went wrong with core initalization!
	exit 1
fi

# First let's create the multiple submission material using our existing sample blocks...
jq -s . int-subsequent-block-1.json int-subsequent-block-2.json int-subsequent-block-3.json > $BLOCKS_PATH

# Now we can submit our sample of multiple block that we made...
$BINARY_PATH submitIntBlocks --file=$BLOCKS_PATH

[[ $(getCoreLatestHostBlockNumber) == $(getExpectedBlockNumber) ]] && result=true || result=false

if [[ $result == true ]]; then
	exit 0
else
	exit 1
fi
