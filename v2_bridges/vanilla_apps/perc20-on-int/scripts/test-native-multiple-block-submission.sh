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

function maybeCleanUpSubmissionFile() {
	if [[ -f $BLOCKS_PATH ]]; then
		rm $BLOCKS_PATH > /dev/null
	fi
}

function cleanUpAndExit() {
	exitCodeBeforeCleanup=$?
	../../scripts/clean-up.sh $BINARY_NAME
	maybeCleanUpSubmissionFile

	if [[ $exitCodeBeforeCleanup == 0 ]]; then
		echo [+] NATIVE multiple block submission test to \'$BINARY_NAME\' core test PASSED!
	else
		echo [-] \'${last_command}\' command failed with exit code $?.
		echo [-] NATIVE block submission test to \'$BINARY_NAME\' core test FAILED!
		exit 1
	fi
}

trap cleanUpAndExit EXIT

echo [+] Testing multiple NATIVE block submission to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME
maybeCleanUpSubmissionFile

getCoreLatestNativeBlockNumber() {
	$BINARY_PATH getEnclaveState | jq .eth.eth_latest_block_number
}

getExpectedBlockNumber() {
	cat ./eth-subsequent-block-3.json | jq .block.number
}

./initialize-eth.sh
./initialize-int.sh

if [[ $(getCoreLatestNativeBlockNumber) == null ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Something went wrong with core initalization!
	exit 1
fi

# First let's create the multiple submission material using our existing sample blocks...
jq -s . eth-subsequent-block-1.json eth-subsequent-block-2.json eth-subsequent-block-3.json > $BLOCKS_PATH

# Now we can submit our sample of multiple block that we made...
$BINARY_PATH submitEthBlocks --file=$BLOCKS_PATH

[[ $(getCoreLatestNativeBlockNumber) == $(getExpectedBlockNumber) ]] && result=true || result=false

if [[ $result == true ]]; then
	exit 0
else
	exit 1
fi
