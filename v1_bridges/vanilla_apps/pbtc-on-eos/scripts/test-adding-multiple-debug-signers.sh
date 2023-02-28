#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh
. ../../scripts/get-sample-debug-signers.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../target/release/$BINARY_NAME"

echo ✔ Testing adding multiple debug signatories to $BINARY_NAME core...

../../scripts/clean-up.sh $BINARY_NAME

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

get_second_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[1].ethAddress'
}

./initialize-eos.sh > /dev/null
./initialize-btc.sh > /dev/null

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds addresses with no private keys as signers and is therefore useless.
$BINARY_PATH debugAddDebugSigners $(getSampleDebugSigners) --sig 0x7819bc31098d45fd47ea73610f91fe539181effb51a0e6d6ceec57293eddfd3c6fc4b9f035f17815cf9e18d479c60aebd814955e3da21248cb15e0a87d48db341b > /dev/null

if [[
	$(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000001\"
	&&
	$(get_second_debug_signer_address) == \"0x0000000000000000000000000000000000000002\"
   ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✔ Adding multiple debug signatories to $BINARY_NAME core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Add debug signatory test to $BINARY_NAME FAILED!
	exit 1
fi
