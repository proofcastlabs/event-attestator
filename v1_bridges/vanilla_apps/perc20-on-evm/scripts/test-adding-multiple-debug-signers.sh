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
./initialize-evm.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds addresses with no private keys as signers and is therefore useless.
$BINARY_PATH debugAddDebugSigners $(getSampleDebugSigners) --sig 0xa85bf78c97e87ac7889a0c6b60db925ef21e46ea7fdb99310e9ea0a95298901d40ad38a7c3728cdd87f799c788b4601d552fdd0cdf6adeb2c7bdd439d083a83a1c

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
