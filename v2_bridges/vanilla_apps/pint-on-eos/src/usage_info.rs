pub const USAGE_INFO: &str = "
❍ Provable pINT-on-EOS Core ❍

    Copyright Provable 2022
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pINT-on-EOS app uses the pTokens core in order to manage the cross-chain conversions between
ERC20 Interim pTokens & the pTokenized equivalents on the EOS blockchain.

❍ Usage ❍

Usage:  pint-on-eos [--help]
        pint-on-eos [--version]
        pint-on-eos getEnclaveState
        pint-on-eos getLatestBlockNumbers
        pint-on-eos submitIntBlock (<blockJson> | --file=<path>)
        pint-on-eos submitEosBlock (<blockJson> | --file=<path>)
        pint-on-eos submitIntBlocks (<blockJson> | --file=<path>)
        pint-on-eos initializeEos [--chainId=<str>] (<eosJson> | --file=<path>)
        pint-on-eos initializeInt (<vaultAddress> | --vaultAddress=<vaultAddress>) (<routerAddress> | --routerAddress=<routerAddress>) (<blockJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        pint-on-eos debugGetAllDbKeys [--sig=<hex>]
        pint-on-eos debugGetKeyFromDb <key> [--sig=<hex>]
        pint-on-eos debugSetIntGasPrice <wei> [--sig=<hex>]
        pint-on-eos debugSetIntAccountNonce <nonce> [--sig=<hex>]
        pint-on-eos debugSetEosAccountNonce <nonce> [--sig=<hex>]
        pint-on-eos debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        pint-on-eos debugAddSupportedToken <ethAddress> [--sig=<hex>]
        pint-on-eos debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        pint-on-eos debugRemoveSupportedToken <ethAddress> [--sig=<hex>]
        pint-on-eos debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        pint-on-eos debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        pint-on-eos debugRemoveDictionaryEntry <ethAddress> [--sig=<hex>]
        pint-on-eos debugEnableEosProtocolFeature <featureHash> [--sig=<hex>]
        pint-on-eos debugDisableEosProtocolFeature <featureHash> [--sig=<hex>]
        pint-on-eos debugUpdateIncremerkle (<eosJson> | --file=<path>) [--sig=<hex>]
        pint-on-eos debugReprocessIntBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-eos debugReprocessEosBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-eos debugAddEosSchedule (<scheduleJson> | --file=<path>) [--sig=<hex>]
        pint-on-eos debugAddDictionaryEntry (<entryJson> | --file=<path>) [--sig=<hex>]
        pint-on-eos debugResetIntChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        pint-on-eos debugReprocessEosBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        pint-on-eos signHexMsgWithIntKeyWithPrefix <message>
        pint-on-eos signAsciiMsgWithIntKeyWithNoPrefix <message>

Commands:

    submitIntBlock                      ❍ Submit an INT block (& its receipts) to the enclave.
                                          ➔ blockJson Format:
                                          {
                                              `block`: The block header itself,
                                              `receipts`: An array containing the block's receipts,
                                              `ref_block_num`: A current EOS reference block number,
                                              `ref_block_prefix`: A current EOS reference block prefix,
                                          }

    submitIntBlocks                     ❍ Submit multiple INT blocks to the enclave.

    submitEosBlock                      ❍ Submit an EOS block (& its receipts) to the enclave.
                                          ➔ blockJson Format:
                                          {
                                              `block_header`: An EOS block header,
                                              `action_proofs`: An array of EOS action proofs,
                                              `interim_block_ids`: An array of EOS block IDs from the core's latest to
                                                                   the block above,
                                          }

    initializeEos                       ❍ Initialize the enclave with the first trusted EOS block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignore by the
                                          enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its EOS
                                          related database from this trusted block, create the EOS private-key and seal
                                          it plus any relevant settings from the `config` into the database. This
                                          command will return a signed transaction to broadcast, which transaction will
                                          deploy the pToken contract to the EOS network.
                                          ➔ blockJson Format:
                                          {
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

    initializeInt                       ❍ Initialize the enclave with the first trusted INT block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignore by the
                                          enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its INT
                                          related database from this trusted block, create the INT private-key and seal
                                          it plus any relevant settings from the `config` into the database. This
                                          command will return a signed transaction to broadcast, which transaction will
                                          deploy the pToken contract to the INT network.
                                          ➔ blockJson Format: See `submitINTBlock` for format of the JSON.

    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugEnableEosProtocolFeature       ❍ Enable an EOS protocol feature in the core.

    debugDisableEosProtocolFeature      ❍ Disable an EOS protocol feature in the core.

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddEosSchedule                 ❍ Add an EOS schedule to the database.

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.

    getLatestBlockNumbers               ❍ Returns the current lastest EOS & EOS block numbers seen by the enclave.

    debugSetIntGasPrice                 ❍ Set the gas price for INT transactions.

    debugResetIntChain                  ❍ Resets the INT chain in the encrypted database using the supplied block as a
                                          new starting point.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can only
                                          be called if the `debug` flag is set to true when the core is built. Note that
                                          there are zero checks on what is passed in to the database: Use at own risk!

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

    debugAddSupportedToken              ❍ Returns a signed transaction adding the supplied <ethAddress> as a supported
                                          token to the 'pINT-on-eos` smart-contract.

    debugRemoveupportedToken            ❍ Returns a signed transaction removing the supplied <ethAddress> as a supported
                                          token from the 'pINT-on-eos` smart-contract.

    debugReprocessIntBlock              ❍ Reprocess an INT block by resubmitting it to the core.

    debugReprocessEosBlock              ❍ Reprocess an EOS block by resubmitting it to the core.

    signHexMsgWithIntKeyWithPrefix      ❍ Signs an ASCII message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithIntKeyWithNoPrefix  ❍ Signs a HEX message with the INT private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    debugSetEosAccountNonce              ❍ Set the EOS account nonce in the encrypted database to the passed in <nonce>.

    debugSetIntAccountNonce              ❍ Set the INT account nonce in the encrypted database to the passed in <nonce>.

    <key>                               ❍ A database key in HEX format.

    <wei>                               ❍ The INT gas price in Wei.

    <value>                             ❍ A database value in HEX format.

    <blockJson>                         ❍ Valid JSON string of EOS or INT block.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <message>                           ❍ A message to be signed.

    <scheduleJson>                      ❍ A valid EOS schedule JSON.

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

    <ethAddress>                        ❍ A valid ethereum address in hex format.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The ETH address of the debug signer,
                                        }

    <featureHash>                       ❍ A hash as a hex string of an EOS protocol feature.

    <name>                              ❍ The name of the debug signer.

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

    --confs=<uint>                      ❍ The number of confirmations required before signing transactions. This
                                           affects the length of chain the light client maintains in the database.
                                           [default: 0]

    --vaultAddress=<hex>                ❍ The INT address of the ERC20 vault smart-contract.

    --routerAddress=<hex>               ❍ The INT address of the interim-chain router smart-contract.

    --chainId=<hex|uint>                ❍ Hex string of the EOS chain ID, or integer for INT chain ID:
                                            1 = Intereum Main-Net (default)
                                            3 = Ropsten Test-Net
                                            4 = Rinkeby Test-Net
                                            42 = Kovan Test-Net
                                          [default: 1]

    --gasPrice=<uint>                   ❍ The gas price to be used in INT transactions.
                                          [default: 20000000000]

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.
";
