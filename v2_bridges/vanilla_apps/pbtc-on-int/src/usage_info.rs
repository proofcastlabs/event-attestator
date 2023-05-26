pub const USAGE_INFO: &str = "
❍ Provable Vanilla pBTC-on-INT App ❍

    Copyright Provable 2021
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pBTC-on-INT app uses the pToken core in order to manage the cross-chain conversions between pBTC
& it's counterpart interim-chain pToken.

❍ Usage ❍

Usage:  pbtc-on-int [--help]
        pbtc-on-int [--version]
        pbtc-on-int getAllUtxos
        pbtc-on-int getEnclaveState
        pbtc-on-int getLatestBlockNumbers
        pbtc-on-int submitIntBlock (<blockJson> | --file=<path>)
        pbtc-on-int submitBtcBlock (<blockJson> | --file=<path>)
        pbtc-on-int submitIntBlocks (<blockJson> | --file=<path>)
        pbtc-on-int initializeBtc (<blockJson> | --file=<path>) [--network=<string>] [--difficulty=<uint>] [--fee=<uint>] [--confs=<uint>]
        pbtc-on-int initializeInt (<blockJson> | --file=<path>) --pTokenAddress=<hex> --routerAddress=<hex> [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        pbtc-on-int debugGetAllDbKeys [--sig=<hex>]
        pbtc-on-int debugClearAllUtxos [--sig=<hex>]
        pbtc-on-int debugSetBtcFee <fee> [--sig=<hex>]
        pbtc-on-int debugGetKeyFromDb <key> [--sig=<hex>]
        pbtc-on-int debugSetIntGasPrice <wei> [--sig=<hex>]
        pbtc-on-int debugWithdrawFees <address> [--sig=<hex>]
        pbtc-on-int debugRemoveUtxo <txId> <vOut> [--sig=<hex>]
        pbtc-on-int debugSetIntAccountNonce <nonce> [--sig=<hex>]
        pbtc-on-int debugSetBtcAccountNonce <nonce> [--sig=<hex>]
        pbtc-on-int debugRemoveDebugSigner <address> [--sig=<hex>]
        pbtc-on-int debugErc777ChangePNetwork <address> [--sig=<hex>]
        pbtc-on-int debugAddDebugSigner <name> <address> [--sig=<hex>]
        pbtc-on-int debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        pbtc-on-int debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        pbtc-on-int debugErc777ProxyChangePNetwork <address> [--sig=<hex>]
        pbtc-on-int debugAddUtxos (<utxosJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-int debugConsolidateUtxos <numUtxos> [--fee=<uint>] [--sig=<hex>]
        pbtc-on-int debugErc777ProxyChangePNetworkByProxy <address> [--sig=<hex>]
        pbtc-on-int debugMaybeAddUtxoToDb (<blockJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-int debugReprocessBtcBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-int debugReprocessIntBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-int debugGetChildPaysForParentTx <txId> <vOut> [--fee=<uint>] [--sig=<hex>]
        pbtc-on-eos debugConsolidateUtxosToAddress <numUtxos> <address>[--fee=<uint>] [--sig=<hex>]
        pbtc-on-int debugResetIntChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        pbtc-on-int debugReprocessBtcBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-int signMessageWithIntKey <message>
        pbtc-on-int signHexMsgWithIntKeyWithPrefix <message>
        pbtc-on-int signAsciiMsgWithIntKeyWithNoPrefix <message>

Commands:

    submitIntBlock                      ❍ Submit an INT block (& its receipts) to the enclave.  NOTE: The enclave must
                                          first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `Block`: The block header itself.
                                            `Receipt`: An array containing the block's receipts,
                                          }

    submitIntBlocks                     ❍ Submit multiple INT blocks to the core.

    submitBtcBlock                      ❍ Submit an BTC block & its transactions to the enclave. The submission material
                                          must also include an array of deposit information for `p2sh` addresses. NOTE:
                                          The enclave must first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `block`: The BTC block in JSON format.
                                            `transactions`: The transactions in HEX format.
                                            `deposit_address_list`: [
                                                {
                                                  `nonce`: An integer nonce.
                                                  `eth_address`: The destination ETH address in hex.
                                                  `btc_deposit_address`: The `p2sh` BTC deposit address.
                                                  `eth_address_and_nonce_hash`: The `sha256d` of `eth_address + nonce`
                                                },
                                            ]
                                          }

    initializeInt                       ❍ Initialize the enclave with the first trusted INT block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its INT
                                          related database from this trusted block, create the INT private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          ➔ blockJson Format: See `submitIntBlock` for breakdown of JSON.

    initializeBtc                       ❍ Initialize the enclave with the first trusted BTC block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its BTC
                                          related database from this trusted block, create the BTC private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          ➔ blockJson Format: See `submitBTCBlock` for breakdown of JSON.

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    getAllUtxos                         ❍ Returns JSON formatted report of all the UTXOs currently held in the DB.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugChangePnetwork                 ❍ Make the core output a tx which when broadcast will change the pNetwork
                                          address in the ERC777 contract.


    debugSetIntGasPrice                 ❍ Set the gas price for INT transactions.

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.

    debugSetBtcFee                      ❍ Sets the BTC fee to use when making transactions.

    signHexMsgWithIntKeyWithPrefix      ❍ Signs an ASCII message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithIntKeyWithNoPrefix  ❍ Signs a HEX message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signMessageWithIntKey               ❍ DEPRECATED! This is an alias for `signAsciiMsgWithIntKeyWithNoPrefix`

    debugClearAllUtxos                  ❍ Clear all the UTXOs set stored inside the database

    debugConsolidateUtxos               ❍ Combines UTXOs into a single tx sent to the enclave address.

    debugConsolidateUtxosToAddress      ❍ Combines UTXOs into a single tx to sent to the given address.

    getLatestBlockNumbers               ❍ Returns the current lastest INT & BTC block numbers seen by the enclave.

    debugMaybeAddUtxoToDb               ❍ Reprocess a BTC block looking for any UTXOs to add to the core.

    debugReprocessBtcBlock              ❍ Submit BTC block submisson material for re-processing.

    debugReprocessIntBlock              ❍ Submit INT block submisson material for re-processing.

    debugResetIntChain                  ❍ Resets the INT chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugWithdrawFees                   ❍ Creates a BTC transaction to the passed in address for the amount of the total
                                          accrued fees in the core.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can only
                                          be called if the `debug` flag is set to true when the core is built. Note that
                                          there are zero checks on what is passed in to the database: Use at own risk!

    debugAddUtxos                       ❍ Adds multiple UTXOs to the core, if they are not already extant. Format of the
                                          JSON is the same as is outputted from the `debugGetAllUtxos` function.

    debugSetIntAccountNonce             ❍ Set the INT account nonce in the encrypted database to the passed in <nonce>.

    debugSetBtcAccountNonce             ❍ Set the BTC account nonce in the encrypted database to the passed in <nonce>.

    <key>                               ❍ A database key in HEX format.

    <wei>                               ❍ The INT gas price in Wei.

    <value>                             ❍ A database value in HEX format.

    <address>                           ❍ A valid Ethereum or Bitcoin address.

    <blockJson>                         ❍ Valid JSON string of INT or BTC block.

    <txId>                              ❍ The transaction ID of a BTC UTXO.

    <vOut>                              ❍ The output of a BTC UTXO to spend.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The INT address of the debug signer,
                                        }

    <numUtxos>                          ❍ The number of UTXOS to attempt to consolidate.

    <utxosJson>                         ❍ Valid JSON string of UTXOs per the format `debugGetAllUtxos` returns.

    <message>                           ❍ A message to be signed.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <amount>                            ❍ Amount to set accrued fees to.

    <fee>                               ❍ The BTC transaction fee, in Satoshis-per-byte.

    <name>                              ❍ The name of the debug signer.

Options:

    --help                              ❍ Show this message.

    --version                           ❍ Returns the core, lib and application versions as well as the application type.

    --file=<path>                       ❍ Path to file containg an INT or BTC block JSON.

    --fee=<uint>                        ❍ BTC fee as measured in Satoshis per byte.
                                          [default: 23]

    --difficulty=<path>                 ❍ The `difficulty` value above which a BTC block's difficulty should be in order
                                          to be considered valid.
                                          [default: 1337]

    --gasPrice=<uint>                   ❍ The gas price to be used in INT transactions.
                                          [default: 20000000000]

    --pTokenAddress=<hex>               ❍ Address of the pToken contract.

    --routerAddress=<hex>               ❍ Address of the router contract.

    --confs=<uint>                      ❍ The number of confirmations required before signing transactions. This affects
                                          the length of chain the light client maintains in the database.
                                          [default: 0]

    --network=<string>                  ❍ Desired BTC network. Use `Bitcoin` for the maine bitcoin network, and use
                                          `Testnet` for the bitcoin public test-net
                                          [default: Bitcoin]

    --chainId=<uint>                    ❍ ID of desired chain for transaction:
                                          1  = Intereum Main-Net (default)
                                          3  = Ropsten Test-Net
                                          4  = Rinkeby Test-Net
                                          42 = Kovan Test-Net
                                          [default: 1]

    --ethNetwork=<str>                  ❍ Transaction network name
                                            - mainnet
                                            - ropsten
                                            - rinkeby
                                            - kovan

    --recipient=<str>                   ❍ Transaction eth address

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.
";
