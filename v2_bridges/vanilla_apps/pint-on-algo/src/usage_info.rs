pub static USAGE_INFO: &str = "
❍ Provable Vanilla pINT-on-ALGO Enclave ❍

    Copyright Provable 2022
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pINT-on-AGLO app uses the pToken core in order to manage the cross-chain conversions between
interim-chain pTokens & their Algorand pTokenized asset counterparts.

❍ Usage ❍

Usage:  pint-on-algo [--help]
        pint-on-algo [--version]
        pint-on-algo getEnclaveState
        pint-on-algo getLatestBlockNumbers
        pint-on-algo submitIntBlock (<blockJson> | --file=<path>)
        pint-on-algo submitIntBlocks (<blockJson> | --file=<path>)
        pint-on-algo submitAlgoBlock (<blockJson> | --file=<path>)
        pint-on-algo encodeAlgoNoteMetadata <metadataChainId> <destinationAddress> [--userData=<hex>]
        pint-on-algo initializeAlgo (<blockJson> | --file=<path>) [--fee=<u64>] [--confs=<uint>] [--genesisId=<string>]
        pint-on-algo initializeInt (<vaultAddress> | --vaultAddress=<vaultAddress>) (<routerAddress> | --routerAddress=<routerAddress>) (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        pint-on-algo debugGetAllDbKeys [--sig=<hex>]
        pint-on-algo debugGetKeyFromDb <key> [--sig=<hex>]
        pint-on-algo debugSetIntGasPrice <gasPrice> [--sig=<hex>]
        pint-on-algo debugSetIntAccountNonce <nonce> [--sig=<hex>]
        pint-on-algo debugSetAlgoAccountNonce <nonce> [--sig=<hex>]
        pint-on-algo debugSetAlgoAccountNonce <nonce> [--sig=<hex>]
        pint-on-algo debugAddSupportedToken <evmAddress> [--sig=<hex>]
        pint-on-algo debugRemoveDebugSigner <evmAddress> [--sig=<hex>]
        pint-on-algo debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        pint-on-algo debugOptInToApp <appId> <firstValid> [--sig=<hex>]
        pint-on-algo debugAddDebugSigner <name> <evmAddress> [--sig=<hex>]
        pint-on-algo debugRemoveDictionaryEntry <evmAddress> [--sig=<hex>]
        pint-on-algo debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        pint-on-algo debugOptInToAsset <assetId> <firstValid> [--sig=<hex>]
        pint-on-algo debugReprocessIntBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-algo debugReprocessAlgoBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-algo debugAddDictionaryEntry (<entryJson> | --file=<path>) [--sig=<hex>]
        pint-on-algo debugResetIntChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        pint-on-algo debugResetAlgoChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        pint-on-algo debugReprocessAlgoBlockWithNonce (<blockJson> | --file=<path>) <nonce> [--sig=<hex>]
        pint-on-algo debugAlgoPayTx <amount> <receiver> <firstValid> [--fee<ualgos>] [--note=<hex>] [--genesisId=<str>] [--sig=<hex>]

Commands:
    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugAlgoPayTx                      ❍ Create a pay transaction signed by the ALGO private key in the encrypted
                                          database.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can
                                          only be called if the `debug` flag is set to true when the core is built.

    debugSetIntGasPrice                 ❍ Sets the INT gas price to use when making INT transactions. (Unit: Wei)

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if
                                          the `debug` flag is set to true when the tool was built.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugSetAlgoAccountNonce            ❍ Sets the Algo account nonce in the database to the passed in nonce.

    debugSetIntAccountNonce             ❍ Sets the INT account nonce in the database to the passed in nonce.

    encodeAlgoNoteMetadata              ❍ Encodes the Algo note metadata requred for a pToken redeem transaction.

    debugReprocessAlgoBlock             ❍ Submit ALGO block submisson material for re-processing.

    debugReprocessAlgoBlockWithNonce    ❍ Submit ALGO block submisson material for re-processing, signing any txs
                                          with the passed in nonce.

    debugReprocessIntBlock              ❍ Submit INT block submisson material for re-processing.

    debugRemoveDictionaryEntry          ❍ Remove a dictionary entry via its EVM address.

    debugAddSupportedToken              ❍ Get a signed transaction which adds the passed in token address as a
                                          supported token in the ERC20 token vault for this core.

    debugAddDictionaryEntry             ❍ Add a dictionary entry.

    debugResetAlgoChain                 ❍ Resets the ALGO chain in the encrypted database using the supplied block
                                          as a new starting point.

    debugResetIntChain                  ❍ Resets the INT chain in the encrypted database using the supplied block
                                          as a new starting point.

    debugOptInToAsset                   ❍ Returns a transaction which allows the enclave ALGO address to receive an
                                          asset with the given asset ID.

    debugOptInToApp                     ❍ Returns a transaction which allows the enclave ALGO address to interact
                                          with the given app.


    getLatestBlockNumbers               ❍ Returns the current lastest INT & ALGO block numbers from the enclave.

    submitAlgoBlock                     ❍ Submit an ALGO block (& its receipts) to the enclave.  NOTE: The enclave
                                          must first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `block`: The algorand block header itself.
                                            `transactions`: An array containing the block's transactions,
                                          }

    submitIntBlock                      ❍ Submit an INT block (& its receipts) to the enclave.  NOTE: The enclave must
                                          first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `block`: The block header itself.
                                            `receipts`: An array containing the block's receipts,
                                          }

    submitIntBlocks                     ❍ Submit multiple INT blocks to the enclave.

    initializeInt                       ❍ Initialize the enclave with the first trusted INT block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its INT
                                          related database from this trusted block, create the INT private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the INT network.
                                          ➔ blockJson Format: See `submitIntBlock` for breakdown of JSON.

    initializeAlgo                      ❍ Initialize the enclave with the first trusted ALGO block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its ALGO
                                          related database from this trusted block, create the ALGO private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          ➔ blockJson Format: See `submitAlgoBlock` for breakdown of JSON.

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    <blockJson>                         ❍ A valid JSON string of INT or an ALGO block.

    <destinationAddress>                ❍ Destination address to deliver assets to on the destination chain.

    <gasPrice>                          ❍ The gas price (in Wei) to when making transactions.

    <assetId>                           ❍ The ID of an Algorand asset.

    <appId>                             ❍ The ID of an Algorand application.

    <metadataChainId>                   ❍ The metadata chain ID of the destination chain.

    <evmAddress>                        ❍ A valid EVM-compliant address in hex format.

    <firstValid>                        ❍ The round/block number from which an Algo transaction may be broadcast.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The INT address of the debug signer,
                                        }

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <value>                             ❍ A database value in HEX format.

    <amount>                            ❍ Amount in uALGOS to transfer.

    <receiver>                          ❍ The receiving address of the ALGO transaction.

    <vaultAddress>                      ❍ The INT address of the ERC20 vault smart-contract.

    <routerAddress>                     ❍ The INT address of the interim-chain router smart-contract.

    <name>                              ❍ The name of the debug signer.

    <entryJson>                         ❍ Valid JSON string of a dictionary entry.
                                          ➔ JSON Format:
                                          {
                                              `evm_address`: The EVM token address.
                                              `algo_asset_id`: The ALGO asset ID.
                                              `evm_decimals`: The number of decimals of the EVM token.
                                              `algo_decimals`: The number of decimals of the ALGO asset.
                                          }

Options:

    --help                              ❍ Show this message.

    --version                           ❍ Returns the core, lib and application versions plus the application type.

    --file=<path>                       ❍ Path to file containg an INT or ALGO block JSON.

    --chainId=<uint>                    ❍ ID of desired chain for transaction [default: 1]

    --gasPrice=<uint>                   ❍ The gas price to be used in INT transactions [default: 20000000000]

    --fee=<u64>                         ❍ The fee to us for Algorand transactions [default: 1000]

    --genesisId=<string>                ❍ The genesis ID of the network to create Algorand txs for [default: mainnet-v1.0]

    --confs=<u64>                       ❍ The number of confirmations required before signing transactions. This affects

    --vaultAddress=<hex>                ❍ The INT address of the ERC20 vault smart-contract.

    --routerAddress=<hex>               ❍ The INT address of the interim-chain router smart-contract.

    --userData=<hex>                    ❍ User data to include with the cross chain transaction [default: 0x]

    --fee=<ualgos>                      ❍ Fee in micro algos [default: 1000]

    --genesisId=<str>                   ❍ The genesis ID of the chain you want to transact on [default: mainnet-v1.0]

    --note=<hex>                        ❍ An optional note to add to Algo transaction [default: 0x]

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.
";
