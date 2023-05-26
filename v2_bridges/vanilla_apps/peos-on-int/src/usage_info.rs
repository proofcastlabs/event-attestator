pub static USAGE_INFO: &str = "
❍ Provable Vanilla pEOS-on-INT App ❍

    Copyright Provable 2021
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pEOS-on-INT app uses the pToken core in order to manage the cross-chain conversions between native
EOS tokens and their pTokenized ethereum counterparts.

❍ Usage ❍

Usage:  peos-on-int [--help]
        peos-on-int [--version]
        peos-on-int getEnclaveState
        peos-on-int getLatestBlockNumbers
        peos-on-int submitIntBlock (<blockJson> | --file=<path>)
        peos-on-int submitEosBlock (<blockJson> | --file=<path>)
        peos-on-int submitIntBlocks (<blockJson> | --file=<path>)
        peos-on-int initializeEos [--accountName=<string>] [--chainId=<hex>] (<eosJson> | --file=<path>)
        peos-on-int initializeInt (<routerAddress> | --routerAddress=<routerAddress>) (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        peos-on-int debugGetAllDbKeys [--sig=<hex>]
        peos-on-int debugGetKeyFromDb <key>  [--sig=<hex>]
        peos-on-int debugSetIntGasPrice <wei>  [--sig=<hex>]
        peos-on-int debugSetIntAccountNonce <nonce>  [--sig=<hex>]
        peos-on-int debugSetEosAccountNonce <nonce>  [--sig=<hex>]
        peos-on-int debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        peos-on-int debugSetKeyInDbToValue <key> <value>  [--sig=<hex>]
        peos-on-int debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        peos-on-int debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        peos-on-int debugRemoveDictionaryEntry <ethAddress>  [--sig=<hex>]
        peos-on-int debugEnableEosProtocolFeature <featureHash>  [--sig=<hex>]
        peos-on-int debugDisableEosProtocolFeature <featureHash>  [--sig=<hex>]
        peos-on-int debugUpdateIncremerkle (<eosJson> | --file=<path>)  [--sig=<hex>]
        peos-on-int debugReprocessIntBlock (<blockJson> | --file=<path>)  [--sig=<hex>]
        peos-on-int debugReprocessEosBlock (<blockJson> | --file=<path>)  [--sig=<hex>]
        peos-on-int debugAddEosSchedule (<scheduleJson> | --file=<path>)  [--sig=<hex>]
        peos-on-int debugAddDictionaryEntry (<entryJson> | --file=<path>)  [--sig=<hex>]
        peos-on-int debugResetIntChain (<blockJson> | --file=<path>) [--confs=<uint>]  [--sig=<hex>]
        peos-on-int debugReprocessEosBlockWithNonce <nonce> (<blockJson> | --file=<path>)  [--sig=<hex>]
        peos-on-int signMessageWithIntKey <message>
        peos-on-int signHexMsgWithIntKeyWithPrefix <message>
        peos-on-int signAsciiMsgWithIntKeyWithNoPrefix <message>

Commands:

    submitIntBlock                      ❍ Submit an INT block (& its receipts) to the enclave.  NOTE: The enclave must
                                          first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `Block`: The block header itself.
                                            `Receipt`: An array containing the block's receipts,
                                          }

    submitIntBlock                      ❍ Submit multiple INT blocks to the enclave.

    submitEosBlock                      ❍ Submit an EOS block (& its receipts) to the enclave.
                                          ➔ blockJson Format:
                                          {
                                              `block_header`: An EOS block header,
                                              `action_proofs`: An array of EOS action proofs,
                                              `interim_block_ids`: An array of EOS block IDs from the core's latest to
                                                                   the block above,
                                          }

    initializeInt                       ❍ Initialize the enclave with the first trusted INT block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its INT
                                          related database from this trusted block, create the INT private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the INT network.
                                          ➔ blockJson Format: See `submitIntBlock` for breakdown of JSON.

    initializeEos                       ❍ Initialize the enclave with the first trusted EOS block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignore by the
                                          enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its EOS
                                          related database from this trusted block, create the EOS private-key and seal
                                          it plus any relevant settings from the `config` into the database. This
                                          command will return a signed transaction to broadcast, which transaction will
                                          deploy the pToken contract to the EOS network.
                                          ➔ blockJson Format:

                                              `block`: An EOS block,
                                              `active_schedule`: The active schedule for the above block,
                                              `blockroot_merkle`: The blockroot-merkles for the above block,
                                              `erc20_on_eos_token_dictionary`: [{
                                                `eos_symbol`: Symbol for the EOS token,
                                                `eth_symbol`: Symbol for the INT token,
                                                `eos_address`: Address of the EOS token,
                                                `eth_address`: Address of the INT token,
                                                `eth_token_decimals`: Number of decimals in the INT token,
                                                `eos_token_decimals`: Number of decimals in the EOS token,
                                              }]
                                          }

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugSetIntGasPrice                 ❍ Set the gas price for INT transactions.

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.


    signHexMsgWithIntKeyWithPrefix      ❍ Signs an ASCII message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithIntKeyWithNoPrefix  ❍ Signs a HEX message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signMessageWithIntKey               ❍ DEPRECATED! This is an alias for `signAsciiMsgWithIntKeyWithNoPrefix`

    getLatestBlockNumbers               ❍ Returns the current lastest INT & BTC block numbers seen by the enclave.

    debugReprocessEosBlock              ❍ Submit BTC block submisson material for re-processing.

    debugReprocessIntBlock              ❍ Submit INT block submisson material for re-processing.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can only
                                          be called if the `debug` flag is set to true when the core is built. Note that
                                          there are zero checks on what is passed in to the database: Use at own risk!

    debugEnableEosProtocolFeature       ❍ Enable an EOS protocol feature in the core.

    debugDisableEosProtocolFeature      ❍ Disable an EOS protocol feature in the core.

    debugAddEosSchedule                 ❍ Add an EOS schedule to the database.

    debugSetIntAccountNonce             ❍ Set the INT account nonce in the encrypted database to the passed in <nonce>.

    debugSetEosAccountNonce             ❍ Set the EOS account nonce in the encrypted database to the passed in <nonce>.

    debugResetIntChain                  ❍ Resets the INT chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugUpdateIncremerkle              ❍ Use a trusted block header, blockroot_merkle and blockroot_merkle to update
                                          the EOS incremerkle in the database, thus effectively moving the chain forward
                                          to the submittied block's height.
                                          ➔ eosJson Format:
                                            {
                                              `block`: An EOS block,
                                              `active_schedule`: The active schedule for the above block,
                                              `blockroot_merkle`: The blockroot-merkles for the above block,
                                            }

    debugAddDictionaryEntry             ❍ Add an `EosErc20DictionaryEntry` to the core's encrypted databsae.

    debugRemoveDictionaryEntry          ❍ Remove an `EosErc20DictionaryEntry` to the core's encrypted databsae.

    debugReprocessEosBlock              ❍ Reprocess an EOS block by resubmitting it to the core.

    <key>                               ❍ A database key in HEX format.

    <value>                             ❍ A database value in HEX format.

    <blockJson>                         ❍ Valid JSON string of INT or BTC block.

    <path>                              ❍ Path to file containing data relevnt to the chosen command.

    <wei>                               ❍ The INT gas price in Wei.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <message>                           ❍ A message to be signed.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The ETH address of the debug signer,
                                        }

    <blockJson>                         ❍ Valid JSON string of EOS or INT block.

    <scheduleJson>                      ❍ A valid EOS schedule JSON.

    <routerAddress>                     ❍ The ETH address of the ERC20 router smart-contract.

    <name>                              ❍ The name of the debug signer.

    <entryJson>                         ❍ Valid JSON string of a dictionary entry.
                                          ➔ JSON Format:
                                          {
                                             `eos_symbol`: The EOS token symbol,
                                             `eth_symbol`: The INT token symbol,
                                             `eos_address`: The EOS token address,
                                             `eth_address`: The INT token address,
                                             `eth_token_decimals`: The number of decimals the INT token has,
                                             `eos_token_decimals`: The number of decimals the EOS token has,
                                          }

    <featureHash>                       ❍ A hash as a hex string of an EOS protocol feature.

    <eosJson>                           ❍ Valid JSON string of an object with the fields:
                                          ➔ JSON Format:
                                          {
                                            `block`: An EOS block,
                                            `active_schedule`: The active schedule for the above block,
                                            `blockroot_merkle`: The blockroot-merkles for the above block,
                                          }

Options:

    --help                              ❍ Show this message.

    --version                           ❍ Returns the core, lib and application versions as well as the application type.

    --file=<path>                       ❍ Path to file containing a JSON relevant to the chosen command.

    --gasPrice=<uint>                   ❍ The gas price to be used in INT transactions.
                                          [default: 20000000000]

    --chainId=<hex|uint>                ❍ Hex string of the EOS chain ID, or integer for ETH chain ID:
                                            1 = Ethereum Main-Net (default)
                                            3 = Ropsten Test-Net
                                            4 = Rinkeby Test-Net
                                            42 = Kovan Test-Net
                                          [default: 1]

    --routerAddress=<hex>               ❍ The INT address of the ERC20 router smart-contract.

    --confs=<uint>                      ❍ The number of confirmations required before signing transactions. This affects
                                          the length of chain the light client maintains in the database.
                                          [default: 0]

    --accountName=<string>              ❍ Account name of the authorized user of the EOS smart contract.
                                          [default: pbtctokenxxx]

    --ethNetwork=<str>                  ❍ Transaction network name
                                            - mainnet
                                            - ropsten
                                            - rinkeby
                                            - kovan

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.
";
