pub static USAGE_INFO: &str = "
❍ Provable Vanilla pEOS-onETH App ❍

    Copyright Provable 2021
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pEOS-on-ETH app uses the pToken core in order to manage the cross-chain conversions between native
EOS tokens and their pTokenized ethereum counterparts.

❍ Usage ❍

Usage:  peos-on-eth [--help]
        peos-on-eth [--version]
        peos-on-eth getEnclaveState
        peos-on-eth getLatestBlockNumbers
        peos-on-eth enableEosProtocolFeature <featureHash>
        peos-on-eth disableEosProtocolFeature <featureHash>
        peos-on-eth submitEthBlock (<blockJson> | --file=<path>)
        peos-on-eth submitEosBlock (<blockJson> | --file=<path>)
        peos-on-eth initializeEos [--accountName=<string>] [--chainId=<hex>] (<eosJson> | --file=<path>)
        peos-on-eth initializeEth (<blocksJson> | --file=<path>) [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        peos-on-eth debugGetAllDbKeys [--sig=<hex>]
        peos-on-eth debugGetKeyFromDb <key> [--sig=<hex>]
        peos-on-eth debugSetEthGasPrice <wei> [--sig=<hex>]
        peos-on-eth debugSetEthAccountNonce <nonce> [--sig=<hex>]
        peos-on-eth debugSetEosAccountNonce <nonce> [--sig=<hex>]
        peos-on-eth debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        peos-on-eth debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        peos-on-eth debugRemoveDictionaryEntry <ethAddress> [--sig=<hex>]
        peos-on-eth debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        peos-on-eth debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        peos-on-eth debugSetAccruedFees <ethAddress> <amount> [--sig=<hex>]
        peos-on-eth debugSetEthFeeBasisPoints <address> <basisPoints> [--sig=<hex>]
        peos-on-eth debugSetEosFeeBasisPoints <address> <basisPoints> [--sig=<hex>]
        peos-on-eth debugUpdateIncremerkle (<eosJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugReprocessEthBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugReprocessEosBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugAddEosSchedule (<scheduleJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugAddDictionaryEntry (<entryJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugResetEthChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        peos-on-eth debugReprocessEthBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugReprocessEosBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugReprocessEosBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        peos-on-eth debugWithdrawFees <ethAddress> <eosAddress> <refBlockNum> <refBlockPrefix> [--sig=<hex>]
        peos-on-eth signMessageWithEthKey <message>
        peos-on-eth signHexMsgWithEthKeyWithPrefix <message>
        peos-on-eth signAsciiMsgWithEthKeyWithNoPrefix <message>

Commands:

    submitEthBlock                      ❍ Submit an ETH block (& its receipts) to the enclave.  NOTE: The enclave must
                                          first have been initialized!
                                          ➔ blockJson Format:
                                          {
                                            `Block`: The block header itself.
                                            `Receipt`: An array containing the block's receipts,
                                          }

    submitEosBlock                      ❍ Submit an EOS block (& its receipts) to the enclave.
                                          ➔ blockJson Format:
                                          {
                                              `block_header`: An EOS block header,
                                              `action_proofs`: An array of EOS action proofs,
                                              `interim_block_ids`: An array of EOS block IDs from the core's latest to
                                                                   the block above,
                                          }

    initializeEth                       ❍ Initialize the enclave with the first trusted ETH block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignored by
                                          the enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its ETH
                                          related database from this trusted block, create the ETH private-key and seal
                                          it plus any relevant settings from the `config` into the database.
                                          This command will return a signed transaction to broadcast, which transaction
                                          will deploy the pToken contract to the ETH network.
                                          ➔ blocksJson Format: See `submitETHBlock` for breakdown of JSON.

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
                                                `eth_symbol`: Symbol for the ETH token,
                                                `eos_address`: Address of the EOS token,
                                                `eth_address`: Address of the ETH token,
                                                `eth_token_decimals`: Number of decimals in the ETH token,
                                                `eos_token_decimals`: Number of decimals in the EOS token,
                                              }]
                                          }

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugSetEthGasPrice                 ❍ Set the gas price for ETH transactions.

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.


    signHexMsgWithEthKeyWithPrefix      ❍ Signs an ASCII message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithEthKeyWithNoPrefix  ❍ Signs a HEX message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signMessageWithEthKey               ❍ DEPRECATED! This is an alias for `signAsciiMsgWithEthKeyWithNoPrefix`

    getLatestBlockNumbers               ❍ Returns the current lastest ETH & BTC block numbers seen by the enclave.

    debugReprocessEosBlock              ❍ Submit BTC block submisson material for re-processing.

    debugReprocessEthBlock              ❍ Submit ETH block submisson material for re-processing.

    debugSetKeyInDbToValue              ❍ Set a given <key> in the database to a given <value>. This function can only
                                          be called if the `debug` flag is set to true when the core is built. Note that
                                          there are zero checks on what is passed in to the database: Use at own risk!

    enableEosProtocolFeature            ❍ Enable an EOS protocol feature in the core.

    disableEosProtocolFeature           ❍ Disable an EOS protocol feature in the core.

    debugRemoveDebugSigner              ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigner                 ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddEosSchedule                 ❍ Add an EOS schedule to the database.

    debugSetEthAccountNonce             ❍ Set the ETH account nonce in the encrypted database to the passed in <nonce>.

    debugSetEosAccountNonce             ❍ Set the EOS account nonce in the encrypted database to the passed in <nonce>.

    debugResetEthChain                  ❍ Resets the ETH chain in the encrypted database using the supplied block as a
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

    debugSetEthFeeBasisPoints           ❍ Set the fee basis points in the `EosEthTokenDictionary` for the entry that
                                          pertains to the passed in ETH address.

    debugSetEosFeeBasisPoints           ❍ Set the fee basis points in the `EosEthTokenDictionary` for the entry that
                                          pertains to the passed in EOS address.

    debugWithdrawFees                   ❍ Withdraw fees for a given ETH token address and send them to the given EOS
                                          recipient address.

    debugSetAccruedFees                 ❍ Sets the accrued fees value in a dictionary entry.

    debugReprocessEthBlockWithFeeAccrual ❍ Submit ETH block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugReprocessEosBlockWithFeeAccrual ❍ Submit EOS block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    <key>                               ❍ A database key in HEX format.

    <value>                             ❍ A database value in HEX format.

    <amount>                            ❍ An amount as a string.

    <blockJson>                         ❍ Valid JSON string of ETH or BTC block.


    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The ETH address of the debug signer,
                                        }

    <ethAddress>                        ❍ A valid ethereum address in hex format.

    <eosAddress>                        ❍ A valid EOS address.

    <refBlockNum>                       ❍ An EOS reference block number, as a u16. You can use the `ptokens-eos-syncer`
                                          to get this via the command: `node eos-syncer.js getRefBlockInfo`.

    <refBlockPrefix>                    ❍ An EOS reference block prefix, as a u32. You can use the `ptokens-eos-syncer`
                                          to get this via the command: `node eos-syncer.js getRefBlockInfo`.

    <address>                           ❍ An ETH or EOS address.

    <basisPoints>                       ❍ Basis points to use as a fee.

    <path>                              ❍ Path to file containing data relevnt to the chosen command.

    <wei>                               ❍ The ETH gas price in Wei.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <message>                           ❍ A message to be signed.

    <blockJson>                         ❍ Valid JSON string of EOS or ETH block.

    <scheduleJson>                      ❍ A valid EOS schedule JSON.

    <entryJson>                         ❍ Valid JSON string of a dictionary entry.
                                          ➔ JSON Format:
                                          {
                                             `eos_symbol`: The EOS token symbol,
                                             `eth_symbol`: The ETH token symbol,
                                             `eos_address`: The EOS token address,
                                             `eth_address`: The ETH token address,
                                             `eth_token_decimals`: The number of decimals the ETH token has,
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

    <name>                              ❍ Name of the debug signer.

Options:

    --help                              ❍ Show this message.

    --version                           ❍ Returns the core, lib and application versions as well as the application type.

    --file=<path>                       ❍ Path to file containing a JSON relevant to the chosen command.

    --gasPrice=<uint>                   ❍ The gas price to be used in ETH transactions.
                                          [default: 20000000000]

    --chainId=<hex|uint>                ❍ Hex string of the EOS chain ID, or integer for ETH chain ID:
                                            1 = Ethereum Main-Net (default)
                                            3 = Ropsten Test-Net
                                            4 = Rinkeby Test-Net
                                            42 = Kovan Test-Net
                                          [default: 1]

    --confs=<uint>                      ❍ The number of confirmations required before signing transactions. This affects
                                          the length of chain the light client maintains in the database.
                                          [default: 0]

    --accountName=<string>              ❍ Account name of the authorized user of the EOS smart contract.
                                          [default: pbtctokenxxx]

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.

    --ethNetwork=<str>                  ❍ Transaction network name
                                            - mainnet
                                            - ropsten
                                            - rinkeby
                                            - kovan
";
