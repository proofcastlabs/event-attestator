#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh
. ../../scripts/get-sample-debug-signers.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../target/release/$BINARY_NAME"

echo ✔ Testing adding multiple debug signatories to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

get_second_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[1].ethAddress'
}

./initialize-eth.sh
./initialize-eos.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds addresses with no private keys as signers and is therefore useless.
$BINARY_PATH debugAddDebugSigners $(getSampleDebugSigners) --sig 0xd9ec94b64cf2b26f4020a414c3fbdf29ed7d0aa225cfdf93898c0d33c11d34c010b1376afd38492e3929966d7784b2048db502e4d3dc01d2b8f4eb33f97891d21c

if [[
	$(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000001\"
	&&
	$(get_second_debug_signer_address) == \"0x0000000000000000000000000000000000000002\"
   ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✔ Adding multiple debug signatories to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Add debug signatory test to \'$BINARY_NAME\' FAILED!
	exit 1
fi
