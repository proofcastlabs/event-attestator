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
./initialize-btc.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds addresses with no private keys as signers and is therefore useless.
$BINARY_PATH debugAddDebugSigners $(getSampleDebugSigners) --sig 0xd6caade2ff2bb2c47428d2854c7a4040a9ef7a1d6956f448ca637e37d06b751518c7d5f84e24a27dd7f26dd7efbb43be8b4c199fcd235bdc9ea65b2dc4e9274d1b

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
