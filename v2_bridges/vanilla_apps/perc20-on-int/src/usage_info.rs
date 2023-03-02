pub static USAGE_INFO: &str = "
❍ Provable Vanilla pERC20-on-INT Enclave ❍

    Copyright Provable 2021
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pERC20-on-INT app uses the pToken core in order to manage the cross-chain conversions between
native ethereum ERC20 tokens and their interim-chain compliant chain pTokenized counterparts.

❍ Usage ❍

Usage:  perc20-on-int [--help]
        perc20-on-int [--version]
        perc20-on-int getEnclaveState
        perc20-on-int getLatestBlockNumbers
        perc20-on-int submitEthBlock (<blockJson> | --file=<path>)
        perc20-on-int submitIntBlock (<blockJson> | --file=<path>)
        perc20-on-int submitEthBlocks (<blockJson> | --file=<path>)
        perc20-on-int submitIntBlocks (<blockJson> | --file=<path>)
        perc20-on-int initializeInt (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        perc20-on-int initializeEth (<vaultAddress> | --vaultAddress=<vaultAddress>) (<routerAddress> | --routerAddress=<routerAddress>) (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        perc20-on-int debugGetAllDbKeys [--sig=<hex>]
        perc20-on-int debugGetKeyFromDb <key> [--sig=<hex>]
        perc20-on-int debugSetEthGasPrice <gasPrice> [--sig=<hex>]
        perc20-on-int debugSetIntGasPrice <gasPrice> [--sig=<hex>]
        perc20-on-int debugSetEthAccountNonce <nonce> [--sig=<hex>]
        perc20-on-int debugSetIntAccountNonce <nonce> [--sig=<hex>]
        perc20-on-int debugAddWEthUnwrapper <ethAddress> [--sig=<hex>]
        perc20-on-int debugAddSupportedToken <ethAddress> [--sig=<hex>]
        perc20-on-int debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        perc20-on-int debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        perc20-on-int debugRemoveSupportedToken <ethAddress> [--sig=<hex>]
        perc20-on-int debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        perc20-on-int debugRemoveDictionaryEntry <ethAddress> [--sig=<hex>]
        perc20-on-int debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        perc20-on-int debugSetFeeBasisPoints <ethAddress> <fee> [--sig=<hex>]
        perc20-on-int debugSetAccruedFees <ethAddress> <amount> [--sig=<hex>]
        perc20-on-int debugWithdrawFees <tokenAddress> <recipientAddress> [--sig=<hex>]
        perc20-on-int debugReprocessEthBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int debugReprocessIntBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int debugAddDictionaryEntry (<entryJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int debugResetEthChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        perc20-on-int debugResetIntChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        perc20-on-int debugReprocessEthBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int debugReprocessIntBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int debugReprocessEthBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int debugReprocessIntBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-int signHexMsgWithEthKeyWithPrefix <message>
        perc20-on-int signHexMsgWithIntKeyWithPrefix <message>
        perc20-on-int signAsciiMsgWithEthKeyWithNoPrefix <message>
        perc20-on-int signAsciiMsgWithIntKeyWithNoPrefix <message>

Commands:

    submitEthBlock                      ❍ Submit an ETH block (& its receipts) to the enclave.  NOTE: The enclave must
                                          first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `block`: The block header itself.
                                            `receipts`: An array containing the block's receipts,
                                          }

    submitEthBlocks                     ❍ Submit multiple ETH blocks to the enclave.

    submitIntBlock                      ❍ Submit an INT block (& its receipts) to the enclave.  NOTE: The enclave must
                                          first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `block`: The block header itself.
                                            `receipts`: An array containing the block's receipts,
                                          }

    submitIntBlocks                     ❍ Submit multiple INT blocks to the enclave

    initializeEth                       ❍ Initialize the enclave with the first trusted ETH block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its ETH
                                          related database from this trusted block, create the ETH private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the ETH network.
                                          ➔ blockJson Format: See `submitEthBlock` for breakdown of JSON.

    initializeInt                       ❍ Initialize the enclave with the first trusted INT block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its ETH
                                          related database from this trusted block, create the ETH private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the ETH network.
                                          ➔ blockJson Format: See `submitIntBlock` for breakdown of JSON.

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    addVaultContractAddress             ❍ Adds the ERC20 vault contract address to the encrypted database.

    addRouterContractAddress             ❍ Adds the router contract address to the encrypted database.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugWithdrawFees                   ❍ Withdraw fees for a given token address and send them to the given recipient
                                          address.

    debugSetEthGasPrice                 ❍ Sets the ETH gas price to use when making ETH transactions. (Unit: Wei)

    debugSetIntGasPrice                 ❍ Sets the INT gas price to use when making ETH transactions. (Unit: Wei)

    debugAddWEthUnwrapper               ❍ Get a signed transaction which will add the passed in ETH address as the
                                          WEth unwrapper contract to a given ERC20 vault.

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.

    debugResetEthChain                  ❍ Resets the ETH chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugResetIntChain                  ❍ Resets the INT chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugSetAccruedFees                 ❍ Sets the accrued fees value in a dictionary entry.

    signHexMsgWithEthKeyWithPrefix      ❍ Signs an ASCII message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signHexMsgWithIntKeyWithPrefix      ❍ Signs an ASCII message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithEthKeyWithNoPrefix  ❍ Signs a HEX message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithIntKeyWithNoPrefix  ❍ Signs a HEX message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    getLatestBlockNumbers               ❍ Returns the current lastest ETH & BTC block numbers seen by the enclave.

    debugReprocessEthBlock              ❍ Submit ETH block submisson material for re-processing.

    debugReprocessIntBlock              ❍ Submit INT block submisson material for re-processing.

    debugReprocessEthBlockWithNonce     ❍ Submit ETH block submisson material for re-processing with a custom nonce.

    debugReprocessIntBlockWithNonce     ❍ Submit INT block submisson material for re-processing with a custom nonce.

    debugReprocessEthBlockWithFeeAccrual ❍ Submit ETH block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugReprocessIntBlockWithFeeAccrual ❍ Submit INT block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can only
                                          be called if the `debug` flag is set to true when the core is built. Note that
                                          there are zero checks on what is passed in to the database: Use at own risk!

    debugAddDictionaryEntry             ❍ Add a dictionary entry.

    debugRemoveDictionaryEntry          ❍ Remove a dictionary entry via its ETH address.

    debugSetEthAccountNonce             ❍ Set the ETH account nonce in the encrypted database to the passed in <nonce>.

    debugSetIntAccountNonce             ❍ Set the INT account nonce in the encrypted database to the passed in <nonce>.

    <key>                               ❍ A database key in HEX format.

    <value>                             ❍ A database value in HEX format.

    <blockJson>                         ❍ Valid JSON string of ETH or BTC block.

    <vaultAddress>                      ❍ The ETH address of the ERC20 vault smart-contract.

    <routerAddress>                     ❍ The ETH address of the interim-chain router smart-contract.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The ETH address of the debug signer,
                                        }

    <path>                              ❍ Path to the file being submitted to the app.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <message>                           ❍ A message to be signed.

    <gasPrice>                          ❍ The gas price (in Wei) to when making transactions.

    <ethAddress>                        ❍ A valid ethereum address in hex format.

    <tokenAddress>                      ❍ A valid ethereum token address.

    <amount>                            ❍ An amount as a string.

    <recipientAddress>                  ❍ A valid ethereum fee recipient address.

    <fee>                               ❍ Fee value in basis points.

    <name>                              ❍ The name of the debug signer.

    <entryJson>                         ❍ Valid JSON string of a dictionary entry.
                                          ➔ JSON Format:
                                          {
                                             `eth_symbol`: The ETH token symbol,
                                             `evm_symbol`: The INT token symbol,
                                             `eth_address`: The ETH token address,
                                             `evm_address`: The INT token address,
                                          }

Options:

    --help                              ❍ Show this message.

    --version                           ❍ Returns the core, lib and application versions as well as the application type.

    --file=<path>                       ❍ Path to file containg an ETH or BTC block JSON.

    --gasPrice=<uint>                   ❍ The gas price to be used in ETH transactions.
                                          [default: 20000000000]

    --confs=<uint>                      ❍ The number of confirmations required before signing transactions. This affects
                                          the length of chain the light client maintains in the database.
                                          [default: 0]

    --chainId=<uint>                    ❍ ID of desired chain for transaction:
                                          1  = Ethereum Main-Net (default)
                                          3  = Ropsten Test-Net
                                          4  = Rinkeby Test-Net
                                          42 = Kovan Test-Net
                                          [default: 1]

    --ethNetwork=<str>                  ❍ Transaction network name
                                            - mainnet
                                            - ropsten
                                            - rinkeby
                                            - kovan

    --gasPrice=<uint>                   ❍ Transaction gas price

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.
";
