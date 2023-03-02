#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh
. ../../scripts/get-sample-debug-signers.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../../../target/release/$BINARY_NAME"

echo [+] Testing adding multiple debug signatories to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

get_second_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[1].ethAddress'
}

./initialize-int.sh
./initialize-eos.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds addresses with no private keys as signers and is therefore useless.
$BINARY_PATH debugAddDebugSigners $(getSampleDebugSigners) --sig 0x791285f6e337c79ec9d9f23d4c145392a58d98e22dc59d58e2f813ab47d577057228950e29b3b6b07342be1a05ee67e4bebb107d4c8f1f071f1d1804f874903c1c

if [[
	$(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000001\"
	&&
	$(get_second_debug_signer_address) == \"0x0000000000000000000000000000000000000002\"
   ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [+] Adding multiple debug signatories to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Add debug signatory test to \'$BINARY_NAME\' FAILED!
	exit 1
fi
