#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../target/release/$BINARY_NAME"

echo ✔ Testing adding debug signatory to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

./initialize-eos.sh
./initialize-btc.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds the zero address as a signer and is therefore useless.
$BINARY_PATH debugAddDebugSigner test 0x0000000000000000000000000000000000000000 --sig 0xa731a399ee048f62f8ed2644c5140d9b11697aa300300859f5bb8b695fb35521615387dd7b3bde1c000d065a35b9529d724d1ad11f6c47c707d20b9a3970f2aa1b

if [[ $(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000000\" ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✔ Adding debug signatory to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Add debug signatory test to \'$BINARY_NAME\' FAILED!
	exit 1
fi
