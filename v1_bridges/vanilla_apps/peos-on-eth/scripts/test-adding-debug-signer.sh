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

./initialize-eth.sh
./initialize-eos.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds the zero address as a signer and is therefore useless.
$BINARY_PATH debugAddDebugSigner test 0x0000000000000000000000000000000000000000 --sig 0x360e55433df4809103dd6dd57cd22825c0acbccd9ccbf6ffc52400395dbb054b2d2d0c0b7d52f6799a7ba4e2600f4d5adc53c1f1426a3d2be52fd3c6797397211b

if [[ $(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000000\" ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✔ Adding debug signatory to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo ✘ Add debug signatory test to \'$BINARY_NAME\' FAILED!
	exit 1
fi
