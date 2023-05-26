pub static USAGE_INFO: &str = "
❍ Provable pBTC-on-EOS Core ❍

    Copyright Provable 2020
    Questions: greg@oraclize.it

❍ Info ❍

This Provable pBTC-on-EOS app uses the pToken core in order to manage the cross-chain conversions between pBTC & BTC
tokens on the EOS blockchain.

❍ Usage ❍

Usage:  pbtc-on-eos [--help]
        pbtc-on-eos [--version]
        pbtc-on-eos getAllUtxos
        pbtc-on-eos getEnclaveState
        pbtc-on-eos getLatestBlockNumbers
        pbtc-on-eos enableEosProtocolFeature <featureHash>
        pbtc-on-eos disableEosProtocolFeature <featureHash>
        pbtc-on-eos submitEosBlock (<blockJson> | --file=<path>)
        pbtc-on-eos submitBtcBlock (<blockJson> | --file=<path>)
        pbtc-on-eos initializeEos <accountName> [--symbol=<string>] [--chainId=<hex>] (<eosJson> | --file=<path>)
        pbtc-on-eos initializeBtc (<blocksJson> | --file=<path>) [--network=<string>] [--difficulty=<uint>] [--fee=<uint>] [--confs=<uint>]
        pbtc-on-eos debugGetAllDbKeys [--sig=<hex>]
        pbtc-on-eos debugSetBtcFee <fee> [--sig=<hex>]
        pbtc-on-eos debugGetKeyFromDb <key> [--sig=<hex>]
        pbtc-on-eos debugWithdrawFees <address> [--sig=<hex>]
        pbtc-on-eos debugRemoveUtxo <txId> <vOut> [--sig=<hex>]
        pbtc-on-eos debugSetPegInFee <basisPoints> [--sig=<hex>]
        pbtc-on-eos debugSetEosAccountNonce <nonce> [--sig=<hex>]
        pbtc-on-eos debugSetBtcAccountNonce <nonce> [--sig=<hex>]
        pbtc-on-eos debugSetPegOutFee <basisPoints> [--sig=<hex>]
        pbtc-on-eos debugRemoveDebugSigner <ethAddress> [--sig=<hex>]
        pbtc-on-eos debugSetKeyInDbToValue <key> <value> [--sig=<hex>]
        pbtc-on-eos debugAddDebugSigners <debugSignersJson> [--sig=<hex>]
        pbtc-on-eos debugAddDebugSigner <name> <ethAddress> [--sig=<hex>]
        pbtc-on-eos debugAddUtxos (<utxosJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugConsolidateUtxos <numUtxos> [--fee=<uint>] [--sig=<hex>]
        pbtc-on-eos debugUpdateIncremerkle (<eosJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugMaybeAddUtxoToDb (<blockJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugAddEosSchedule (<scheduleJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugReprocessBtcBlock (<blocksJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugReprocessEosBlock (<blocksJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugGetChildPaysForParentTx <txId> <vOut> [--fee=<uint>] [--sig=<hex>]
        pbtc-on-eos debugConsolidateUtxosToAddress <numUtxos> <address>[--fee=<uint>] [--sig=<hex>]
        pbtc-on-eos debugReprocessBtcBlockAccruingFees (<blocksJson> | --file=<path>) [--sig=<hex>]
        pbtc-on-eos debugReprocessEosBlockAccruingFees (<blocksJson> | --file=<path>) [--sig=<hex>]

Commands:

    submitEosBlock            ❍ Submit an EOS block (& its receipts) to the enclave.  NOTE: The enclave must first have
                                been initialized!
                                ➔ blockJson Format:
                                {
                                  `block_header`: An EOS block header,
                                  `action_proofs`: An array of EOS action proofs,
                                  `interim_block_ids`: An array of block IDs from the core's latest to the block above,
                                }

    submitBtcBlock            ❍ Submit an BTC block & its transactions to the enclave. The submission material must also
                                include an array of deposit information for `p2sh` addresses. NOTE: The enclave must
                                first have been initialized!
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

    initializeEos             ❍ Initialize the enclave with the first trusted EOS block. Ensure the block has NO
                                transactions relevant to the pToken in it, because they'll be ignore by the enclave.
                                Transactions are not verified so you may omit them and include an empty array in their
                                place if needs be. The enclave will initialize its EOS related database from this
                                trusted block, create the EOS private-key and seal it plus any relevant settings from
                                the `config` into the database. This command will return a signed transaction to
                                broadcast, which transaction will deploy the pToken contract to the EOS network.
                                ➔ eosJson Format:
                                {
                                  `block`: An EOS block,
                                  `active_schedule`: The active schedule for the above block,
                                  `blockroot_merkle`: The blockroot-merkles for the above block,
                                }

    initializeBtc             ❍ Initialize the enclave with the first trusted BTC block. Ensure the block has NO
                                transactions relevant to the pToken in it, because they'll be ignore by the enclave.
                                Transactions are not verified so you may omit them and include an empty array in their
                                place if needs be. The enclave will initialize its BTC related database from this
                                trusted block, create the BTC private-key and seal it plus any relevant settings into
                                the encrypted database.
                                ➔ blocksJson Format:
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

    getEnclaveState           ❍ Returns the current state of the enclave as pulled from the database.

    debugConsolidateUtxos     ❍ Combines UTXOs into a single tx sent to the enclave address.

    debugConsolidateUtxosToAddress  ❍ Combines UTXOs into a single tx to sent to the given address.

    getAllUtxos               ❍ Returns JSON formatted report of all the UTXOs currently held in the DB.

    debugGetAllDbKeys         ❍ Returns JSON formatted report of all the database keys used in the core.

    debugRemoveDebugSigner    ❍ Removes a new debug signer to the list stored in the encrypted database.

    debugAddDebugSigner       ❍ Adds a new debug signer to the list stored in the encrypted database.

    debugAddEosSchedule       ❍ Add an EOS schedule to the database.

    debugSetBtcFee            ❍ Sets the BTC fee to use when making transactions.

    debugAddDebugSigners      ❍ Add multiple new debug signers to the core.

    debugAddUtxos             ❍ Adds multiple UTXOs to the core, if they are not already extant. Format of the JSON is
                                the same as is outputted from the `debugGetAllUtxos` function.

    debugMaybeAddUtxoToDb     ❍ Reprocess a BTC block looking for any UTXOs to add to the core.

    debugSetPegInFee          ❍ Sets the fee basis points to use when calculating peg-in fees.

    debugSetPegOutFee         ❍ Sets the fee basis points to use when calculating peg-out fees.

    debugGetKeyFromDb         ❍ Get a given <key> from the database. This function can only be called if the `debug`
                                flag is set to true when the tool was built.

    getLatestBlockNumbers     ❍ Returns the current lastest EOS & BTC block numbers seen by the enclave.

    enableEosProtocolFeature  ❍ Enable an EOS protocol feature in the core.

    disableEosProtocolFeature ❍ Disable an EOS protocol feature in the core.

    debugSetKeyInDbToValue    ❍ Set a given <key> in the database to a given <value>. This function can only be called
                                if the `debug` flag is set to true when the core is built. Note there there are zero
                                checks on what is passed in to the database: Use at own risk!

    debugUpdateIncremerkle    ❍ Use a trusted block header, blockroot_merkle and blockroot_merkle to udpate the EOS
                                incremerkle in the database, thus effectively moving the chain forward to the
                                submittied block's height.
                                ➔ eosJson Format:
                                {
                                  `block`: An EOS block,
                                  `active_schedule`: The active schedule for the above block,
                                  `blockroot_merkle`: The blockroot-merkles for the above block,
                                }

    debugWithdrawFees         ❍ Creates a BTC transaction to the passed in address for the amount of the total accrued
                                fees in the core.

    debugReprocessBtcBlock    ❍ Re-process a BTC block without updating any chain data in the database.

    debugReprocessEosBlock    ❍ Re-process an EOS block.

    debugReprocessBtcBlockAccruingFees ❍ Re-process a BTC block, add any fees to the value stored in the database.

    debugReprocessEosBlockAccruingFees ❍ Re-process an EOS block, adding any fees to the value stored in the database.

    debugSetEosAccountNonce            ❍ Set the EOS account nonce in the encrypted database to the passed in <nonce>.

    debugSetBtcAccountNonce            ❍ Set the BTC account nonce in the encrypted database to the passed in <nonce>.

    <key>                     ❍ A database key in HEX format.

    <value>                   ❍ A database value in HEX format.

    <blockJson>               ❍ Valid JSON string of EOS or BTC block.

    <featureHash>             ❍ A hash as a hex string of an EOS protocol feature.

    <txId>                    ❍ The transaction ID of a BTC UTXO.

    <basisPoints>             ❍ Fee amount in basis points.

    <fee>                     ❍ The BTC transaction fee, in Satoshis-per-byte.

    <vOut>                    ❍ The output of a BTC UTXO to spend.

    <numUtxos>                ❍ The number of UTXOS to attempt to consolidate.

    <utxosJson>               ❍ Valid JSON string of UTXOs per the format `debugGetAllUtxos` returns.

    <nonce>                   ❍ A nonce (as a 64 bit, unsigned integer).

    <eosJson>                 ❍ Valid JSON string of an object with the fields:
                              {
                                `block`: An EOS block,
                                `active_schedule`: The active schedule for the above block,
                                `blockroot_merkle`: The blockroot-merkles for the above block,
                              }

    <accountName>             ❍ Account name of the authorized user of the EOS smart contract.

    <address>                 ❍ A valid bitcoin address.

    <ethAddress>              ❍ A valid ethereum address.

    <name>                    ❍ Name of the debug signer.

    <debugSignersJson>      ❍ Json array of debug signers objects with the fields:
                            {
                              `name`: The name of the debug signer,
                              `eth_address`: The ETH address of the debug signer,
                            }

Options:

    --help                    ❍ Show this message.

    --version                 ❍ Returns the core, lib and application versions as well as the application type.

    --file=<path>             ❍ Path to file containing a JSON relevant to the chosen command.

    --fee=<uint>              ❍ BTC fee as measured in Satoshis per byte.
                                [default: 23]

    --difficulty=<path>       ❍ The `difficulty` value above which a BTC block's difficulty should be in order to be
                                considered valid.
                                [default: 1337]

    --confs=<uint>            ❍ The number of confirmations required before signing transactions. This directly affects
                                the length of chain the light client maintains in the database.
                                [default: 0]

    --network=<string>        ❍ Desired BTC network:
                                Bitcoin = Bitcoin Mainnet (default)
                                Testnet  = Bitcoin public testnet
                                [default: Bitcoin]

    --chainId=<hex>           ❍ Hex string of the EOS chain ID.

    --sig=<hex>               ❍ A signature over the encoded debug command you want to run, in hex format.

    --symbol=<string>         ❍ The symbol of the token.
                                [default: PBTC]
";
