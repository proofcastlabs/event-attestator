#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../../../target/release/$BINARY_NAME"

echo [+] Testing adding debug signatory to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

get_first_debug_signer_address() {
	$BINARY_PATH getEnclaveState | jq '.info.debug_signatories[0].ethAddress'
}

./initialize-int.sh
./initialize-eos.sh

if [[ $(get_first_debug_signer_address) != null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Unexpected debug signatories before the test was started!
	exit 1
fi

# NOTE: This valid signature only adds the zero address as a signer and is therefore useless.
$BINARY_PATH debugAddDebugSigner test 0x0000000000000000000000000000000000000000 --sig 0x6dd76b977b09ee3064fa8748574b3279fc3533a786acfcc969e4225789c7ffaa3f85649547c88f482b18608972b1144033a3d3058901783147e49bc191c50e191c

if [[ $(get_first_debug_signer_address) == \"0x0000000000000000000000000000000000000000\" ]]; then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [+] Adding debug signatory to \'$BINARY_NAME\' core test passed!
else
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Add debug signatory test to \'$BINARY_NAME\' FAILED!
	exit 1
fi
