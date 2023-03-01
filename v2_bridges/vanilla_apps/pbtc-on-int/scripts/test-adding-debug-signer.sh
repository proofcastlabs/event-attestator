#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../../../target/release/$BINARY_NAME"

echo ✔ Testing adding debug signatory to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

./initialize-int.sh
./initialize-btc.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds the zero address as a signer and is therefore useless.
$BINARY_PATH debugAddDebugSigner test 0x0000000000000000000000000000000000000000 --sig 0x739be7f24de6fbe23986594aff2a8b2e68b33c296eaf5934d5546054d217c79614223710e5b5f9e374ae0251c97c354bb422ee2a83a4e74fd87e67cf288ad2551c

if [[ $(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000000\" ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✔ Adding debug signatory to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Add debug signatory test to \'$BINARY_NAME\' FAILED!
	exit 1
fi
