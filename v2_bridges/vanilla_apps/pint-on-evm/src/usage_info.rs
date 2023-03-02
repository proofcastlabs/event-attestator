pub static USAGE_INFO: &str = "
❍ Provable Vanilla pINT-on-EVM Enclave ❍

    Copyright Provable 2021
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pINT-on-EVM app uses the pToken core in order to manage the cross-chain conversions between
interim-chain pTokens & their native ethereum ERC20 tokens pTokenized counterparts.

❍ Usage ❍

Usage:  pint-on-evm [--help]
        pint-on-evm [--version]
        pint-on-evm getEnclaveState
        pint-on-evm getLatestBlockNumbers
        pint-on-evm submitIntBlock (<blockJson> | --file=<path>)
        pint-on-evm submitEvmBlock (<blockJson> | --file=<path>)
        pint-on-evm submitIntBlocks (<blockJson> | --file=<path>)
        pint-on-evm submitEvmBlocks (<blockJson> | --file=<path>)
        pint-on-evm initializeEvm (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        pint-on-evm initializeInt (<vaultAddress> | --vaultAddress=<vaultAddress>) (<routerAddress> | --routerAddress=<routerAddress>) (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        pint-on-evm debugGetAllDbKeys [--sig=<hex>]
        pint-on-evm debugGetKeyFromDb <key> [--sig=<hex>]
        pint-on-evm debugSetIntGasPrice <gasPrice> [--sig=<hex>]
        pint-on-evm debugSetEvmGasPrice <gasPrice> [--sig=<hex>]
        pint-on-evm debugSetIntAccountNonce <nonce> [--sig=<hex>]
        pint-on-evm debugSetEvmAccountNonce <nonce> [--sig=<hex>]
        pint-on-evm debugAddSupportedToken <ethAddress> [--sig=<hex>]
        pint-on-evm debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        pint-on-evm debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        pint-on-evm debugRemoveSupportedToken <ethAddress> [--sig=<hex>]
        pint-on-evm debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        pint-on-evm debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        pint-on-evm debugRemoveDictionaryEntry <ethAddress> [--sig=<hex>]
        pint-on-evm debugSetFeeBasisPoints <ethAddress> <fee> [--sig=<hex>]
        pint-on-evm debugSetAccruedFees <ethAddress> <amount> [--sig=<hex>]
        pint-on-evm debugWithdrawFees <tokenAddress> <recipientAddress> [--sig=<hex>]
        pint-on-evm debugReprocessIntBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm debugReprocessEvmBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm debugAddDictionaryEntry (<entryJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm debugResetIntChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        pint-on-evm debugResetEvmChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        pint-on-evm debugReprocessIntBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm debugReprocessEvmBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm debugReprocessIntBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm debugReprocessEvmBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-evm signHexMsgWithIntKeyWithPrefix <message>
        pint-on-evm signHexMsgWithEvmKeyWithPrefix <message>
        pint-on-evm signAsciiMsgWithIntKeyWithNoPrefix <message>
        pint-on-evm signAsciiMsgWithEvmKeyWithNoPrefix <message>

Commands:

    submitIntBlock                       ❍ Submit an ETH block (& its receipts) to the enclave.  NOTE: The enclave must
                                           first have been initialized!
                                           ➔ blockJson Format:
                                           {
                                             `block`: The block header itself.
                                             `receipts`: An array containing the block's receipts,
                                           }

    submitIntBlocks                      ❍ Submit multiple, contiguous INT blocks, as a JSON array of `<blockJson>`s.

    submitEvmBlock                       ❍ Submit an EVM block (& its receipts) to the enclave.  NOTE: The enclave must
                                           first have been initialized!
                                           ➔ blockJson Format:
                                           {
                                             `block`: The block header itself.
                                             `receipts`: An array containing the block's receipts,
                                           }

    submitEvmBlocks                      ❍ Submit multiple, contiguous EVM blocks, as a JSON array of `<blockJson>`s.

    initializeInt                        ❍ Initialize the enclave with the first trusted INT block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its INT
                                          related database from this trusted block, create the INT private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the INT network.
                                          ➔ blockJson Format: See `submitIntBlock` for breakdown of JSON.

    initializeEvm                       ❍ Initialize the enclave with the first trusted EVM block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its ETH
                                          related database from this trusted block, create the ETH private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the ETH network.
                                          ➔ blockJson Format: See `submitEvmBlock` for breakdown of JSON.

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    addVaultContractAddress             ❍ Adds the ERC20 vault contract address to the encrypted database.

    addRouterContractAddress            ❍ Adds the router contract address to the encrypted database.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugWithdrawFees                   ❍ Withdraw fees for a given token address and send them to the given recipient
                                          address.

    debugSetIntGasPrice                 ❍ Sets the ETH gas price to use when making ETH transactions. (Unit: Wei)

    debugSetEvmGasPrice                 ❍ Sets the EVM gas price to use when making ETH transactions. (Unit: Wei)

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.

    debugResetIntChain                  ❍ Resets the ETH chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugResetEvmChain                  ❍ Resets the EVM chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugSetAccruedFees                 ❍ Sets the accrued fees value in a dictionary entry.

    signHexMsgWithIntKeyWithPrefix      ❍ Signs an ASCII message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signHexMsgWithEvmKeyWithPrefix      ❍ Signs an ASCII message with the EVM private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithIntKeyWithNoPrefix  ❍ Signs a HEX message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithEvmKeyWithNoPrefix  ❍ Signs a HEX message with the EVM private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    getLatestBlockNumbers               ❍ Returns the current lastest ETH & BTC block numbers seen by the enclave.

    debugReprocessIntBlock              ❍ Submit ETH block submisson material for re-processing.

    debugReprocessEvmBlock              ❍ Submit EVM block submisson material for re-processing.

    debugReprocessIntBlockWithNonce     ❍ Submit ETH block submisson material for re-processing with given nonce.

    debugReprocessEvmBlockWithNonce     ❍ Submit EVM block submisson material for re-processing with given nonce.

    debugReprocessIntBlockWithFeeAccrual ❍ Submit ETH block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugReprocessEvmBlockWithFeeAccrual ❍ Submit EVM block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can only
                                          be called if the `debug` flag is set to true when the core is built. Note that
                                          there are zero checks on what is passed in to the database: Use at own risk!

    debugAddDictionaryEntry             ❍ Add a dictionary entry.

    debugRemoveDictionaryEntry          ❍ Remove a dictionary entry via its ETH address.

    debugSetIntAccountNonce             ❍ Set the ETH account nonce in the encrypted database to the passed in <nonce>.

    debugSetEvmAccountNonce             ❍ Set the EVM account nonce in the encrypted database to the passed in <nonce>.

    <key>                               ❍ A database key in HEX format.

    <value>                             ❍ A database value in HEX format.

    <blockJson>                         ❍ Valid JSON string of an INT or EVM block.

    <vaultAddress>                      ❍ The ETH address of the ERC20 vault smart-contract.

    <routerAddress>                     ❍ The ETH address of the interim-chain router smart-contract.

    <path>                              ❍ Path to the file being submitted to the app.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <message>                           ❍ A message to be signed.

    <gasPrice>                          ❍ The gas price (in Wei) to when making transactions.

    <ethAddress>                        ❍ A valid ethereum address in hex format.

    <tokenAddress>                      ❍ A valid ethereum token address.

    <amount>                            ❍ An amount as a string.

    <recipientAddress>                  ❍ A valid ethereum fee recipient address.

    <fee>                               ❍ Fee value in basis points.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The ETH address of the debug signer,
                                        }

    <name>                              ❍ The name of the debug signer.

    <entryJson>                         ❍ Valid JSON string of a dictionary entry.
                                          ➔ JSON Format:
                                          {
                                             `eth_symbol`: The ETH token symbol,
                                             `evm_symbol`: The EVM token symbol,
                                             `eth_address`: The ETH token address,
                                             `evm_address`: The EVM token address,
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
