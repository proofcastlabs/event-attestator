#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../target/release/$BINARY_NAME"

../../scripts/clean-up.sh $BINARY_NAME

echo ✔ Testing adding debug signatory to \'$BINARY_NAME\' core...

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

./initialize-eth.sh
./initialize-btc.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds the zero address as a signer and is therefore useless.
$BINARY_PATH debugAddDebugSigner test 0x0000000000000000000000000000000000000000 --sig 0xae82b1ddc6053d807464e4c6f1e8610fc35275af87c9f821c7862dcf8375c76e4b6ed8fb6888442cc9a53698698af303dd59f72d04dd15787833696a3fb458e81b

if [[ $(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000000\" ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✔ Adding debug signatory to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Add debug signatory test FAILED!
	exit 1
fi
