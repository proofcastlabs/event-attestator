pub const USAGE_INFO: &str = "
❍ Provable pERC20-on-EOS Core ❍

    Copyright Provable 2020
    Questions: greg@oraclize.it

❍ Info ❍

This Provable vanilla pERC20-on-EOS app uses the pTokens core in order to manage the cross-chain conversions between
ERC20 ethereum tokens & the pTokenized equivalents on the EOS blockchain.

❍ Usage ❍

Usage:  perc20-on-eos [--help]
        perc20-on-eos [--version]
        perc20-on-eos getEnclaveState
        perc20-on-eos getLatestBlockNumbers
        perc20-on-eos addVaultContractAddress <ethAddress>
        perc20-on-eos enableEosProtocolFeature <featureHash>
        perc20-on-eos disableEosProtocolFeature <featureHash>
        perc20-on-eos submitEthBlock (<blockJson> | --file=<path>)
        perc20-on-eos submitEosBlock (<blockJson> | --file=<path>)
        perc20-on-eos initializeEos [--chainId=<str>] (<eosJson> | --file=<path>)
        perc20-on-eos initializeEth (<blockJson> | --file=<path>) [--vaultAddress=<hex>] [--chainId=<uint>] [--gasPrice=<uint>] [--confs=<uint>]
        perc20-on-eos debugGetAllDbKeys [--sig=<hex>]
        perc20-on-eos debugGetKeyFromDb <key> [--sig=<hex>]
        perc20-on-eos debugSetEthGasPrice <wei> [--sig=<hex>]
        perc20-on-eos debugSetEthAccountNonce <nonce> [--sig=<hex>]
        perc20-on-eos debugSetEosAccountNonce <nonce> [--sig=<hex>]
        perc20-on-eos debugMigrateContract <ethAddress> [--sig=<hex>]
        perc20-on-eos debugAddSupportedToken <ethAddress> [--sig=<hex>]
        perc20-on-eos debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        perc20-on-eos debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        perc20-on-eos debugRemoveSupportedToken <ethAddress> [--sig=<hex>]
        perc20-on-eos debugRemoveDictionaryEntry <ethAddress> [--sig=<hex>]
        perc20-on-eos debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        perc20-on-eos debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        perc20-on-eos debugSetAccruedFees <ethAddress> <amount> [--sig=<hex>]
        perc20-on-eos debugMigrateSingle <ethAddress> <tokenAddress> [--sig=<hex>]
        perc20-on-eos debugSetEthFeeBasisPoints <address> <basisPoints> [--sig=<hex>]
        perc20-on-eos debugSetEosFeeBasisPoints <address> <basisPoints> [--sig=<hex>]
        perc20-on-eos debugUpdateIncremerkle (<eosJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugWithdrawFees <tokenAddress> <recipientAddress> [--sig=<hex>]
        perc20-on-eos debugReprocessEthBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugReprocessEosBlock (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugAddEosSchedule (<scheduleJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugAddDictionaryEntry (<entryJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugResetEthChain (<blockJson> | --file=<path>) [--confs=<uint>] [--sig=<hex>]
        perc20-on-eos debugReprocessEthBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugReprocessEosBlockWithFeeAccrual (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos debugReprocessEosBlockWithNonce <nonce> (<blockJson> | --file=<path>) [--sig=<hex>]
        perc20-on-eos signMessageWithEthKey <message>
        perc20-on-eos signHexMsgWithEthKeyWithPrefix <message>
        perc20-on-eos signAsciiMsgWithEthKeyWithNoPrefix <message>

Commands:

    submitEthBlock                      ❍ Submit an ETH block (& its receipts) to the enclave.
                                          ➔ blockJson Format:
                                          {
                                              `block`: The block header itself,
                                              `receipts`: An array containing the block's receipts,
                                              `ref_block_num`: A current EOS reference block number,
                                              `ref_block_prefix`: A current EOS reference block prefix,
                                          }

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
                                                `eth_symbol`: Symbol for the ETH token,
                                                `eos_address`: Address of the EOS token,
                                                `eth_address`: Address of the ETH token,
                                                `eth_token_decimals`: Number of decimals in the ETH token,
                                                `eos_token_decimals`: Number of decimals in the EOS token,
                                              }]
                                          }

    initializeEth                       ❍ Initialize the enclave with the first trusted ETH block. Ensure the block has
                                          NO transactions relevant to the pToken in it, because they'll be ignore by the
                                          enclave. Transactions are not verified so you may omit them and include an
                                          empty array in their place if needs be. The enclave will initialize its ETH
                                          related database from this trusted block, create the ETH private-key and seal
                                          it plus any relevant settings from the `config` into the database. This
                                          command will return a signed transaction to broadcast, which transaction will
                                          deploy the pToken contract to the ETH network.
                                          ➔ blockJson Format: See `submitETHBlock` for format of the JSON.

    addVaultContractAddress             ❍ Adds the ERC20 vault contract address to the encrypted database.

    enableEosProtocolFeature            ❍ Enable an EOS protocol feature in the core.

    disableEosProtocolFeature           ❍ Disable an EOS protocol feature in the core.

    getEnclaveState                     ❍ Returns the current state of the enclave as pulled from the database.

    debugGetAllDbKeys                   ❍ Returns JSON formatted report of all the database keys used in the core.

    debugAddDebugSigners                ❍ Add multiple new debug signers to the core.

    debugAddEosSchedule                 ❍ Add an EOS schedule to the database.

    debugGetKeyFromDb                   ❍ Get a given <key> from the database. This function can only be called if the
                                          `debug` flag is set to true when the tool was built.

    getLatestBlockNumbers               ❍ Returns the current lastest EOS & EOS block numbers seen by the enclave.

    debugMigrateContract                ❍ Create a transaction that will migrate then current balances held by the
                                          `pERC20-on-EOS` smart-contract to the `<ethAddress>` supplied.

    debugSetEthGasPrice                 ❍ Set the gas price for ETH transactions.

    debugResetEthChain                  ❍ Resets the ETH chain in the encrypted database using the supplied block as a
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

    debugMigrateSingle                  ❍ Migrates a single token address from the vault to the passed in EVM address.

    debugAddSupportedToken              ❍ Returns a signed transaction adding the supplied <ethAddress> as a supported
                                          token to the 'pERC20-on-eos` smart-contract.

    debugRemoveupportedToken            ❍ Returns a signed transaction removing the supplied <ethAddress> as a supported
                                          token from the 'pERC20-on-eos` smart-contract.

    debugReprocessEthBlock              ❍ Reprocess an ETH block by resubmitting it to the core.

    debugReprocessEosBlock              ❍ Reprocess an EOS block by resubmitting it to the core.

    signHexMsgWithEthKeyWithPrefix      ❍ Signs an ASCII message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and NO prefix is
                                          prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signAsciiMsgWithEthKeyWithNoPrefix  ❍ Signs a HEX message with the ETH private key from the encrypted database.
                                          The message is signed via the `secp256k1` signature scheme and the standard
                                          ethereum-specific prefix IS prepended.
                                          Returns: { message: <inputted-message>, signature: <signature> }

    signMessageWithEthKey               ❍ DEPRECATED! This is an alias for `signAsciiMsgWithEthKeyWithNoPrefix`

    debugSetEthFeeBasisPoints           ❍ Set the fee basis points in the `EosEthTokenDictionary` for the entry that
                                          pertains to the passed in ETH address.

    debugSetEosFeeBasisPoints           ❍ Set the fee basis points in the `EosEthTokenDictionary` for the entry that
                                          pertains to the passed in EOS address.

    debugWithdrawFees                   ❍ Withdraw fees for a given token address and send them to the given recipient
                                          address.

    debugSetAccruedFees                 ❍ Sets the accrued fees value in a dictionary entry.

    debugReprocessEthBlockWithFeeAccrual ❍ Submit ETH block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugReprocessEosBlockWithFeeAccrual ❍ Submit EOS block submisson material for re-processing, adding to accrued fees
                                           whilst doing so.

    debugSetEosAccountNonce              ❍ Set the EOS account nonce in the encrypted database to the passed in <nonce>.

    debugSetEthAccountNonce              ❍ Set the ETH account nonce in the encrypted database to the passed in <nonce>.

    debugRemoveDebugSigner               ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigner                  ❍ Adds a new debug signer to the list stored in the encrypted database.

    <key>                               ❍ A database key in HEX format.

    <wei>                               ❍ The ETH gas price in Wei.

    <value>                             ❍ A database value in HEX format.

    <address>                           ❍ An ETH or EOS address.

    <debugSignersJson>                  ❍ Json array of debug signers objects with the fields:
                                        {
                                          `name`: The name of the debug signer,
                                          `eth_address`: The ETH address of the debug signer,
                                        }

    <basisPoints>                       ❍ Basis points to use as a fee.

    <blockJson>                         ❍ Valid JSON string of EOS or ETH block.

    <nonce>                             ❍ A nonce (as a 64 bit, unsigned integer).

    <amount>                            ❍ An amount as a string.

    <tokenAddress>                      ❍ An ETH token address that exists in the `EosEthTokenDictionary`.

    <recipientAddress>                  ❍ An ETH address to send the withdrawn fees to.

    <message>                           ❍ A message to be signed.

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

    <ethAddress>                        ❍ A valid ethereum address in hex format.

    <featureHash>                       ❍ A hash as a hex string of an EOS protocol feature.

    <name>                              ❍ Name of the debug signer.

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

    --chainId=<hex|uint>                ❍ Hex string of the EOS chain ID, or integer for ETH chain ID:
                                            1 = Ethereum Main-Net (default)
                                            3 = Ropsten Test-Net
                                            4 = Rinkeby Test-Net
                                            42 = Kovan Test-Net
                                          [default: 1]

    --sig=<hex>                         ❍ A signature over the encoded debug command you want to run, in hex format.
    --vaultAddress=<hex>                ❍  The ETH address of the pToken vault.

    --gasPrice=<uint>                   ❍ The gas price to be used in ETH transactions.
                                          [default: 20000000000]
";
