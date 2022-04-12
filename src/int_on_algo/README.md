# :book: `int-on-algo` Core

## :ledger: Algorand Block Validation

When initializing the Algorand side of this core an Algorand block is required. This block must be from a trusted source, since the core cannot verify anything pertaining to it. This block must also not contain any `pToken` related transactions, since they won't be processed.

After the core is initialized, subsequent Algorand blocks submitted to it will cause the following validation to occur:

 - The submitted block's round number is checked to determine whether it is subsequent to the `latest` block stored in the core's database.

 - If that passes, the previous block's header hash is calculated, and compared to the current block's `previous_hash` field.

 - Should this match, the previous block is considered valid and transactions may be written based on any transaction proofs included with it, once the block becomes `canon`.

__NOTE:__ Because Algorand block headers do not commit to the hash of that header, and instead only have a commitment to the _previous_ block's header hash, the `canon-to-tip` length of an Algorand core (or, the number of confirmations an Algorand core needs before writing a `pToken` transaction) must always be greater than or equal to 1. This is enforced in the Algorand initialization logic.

__NOTE:__ Due to Algorand's unforkable nature, forks are not anticipated. Despite this, the underlying logic in the light-client in this codebase allow for `> 1` `canon-to-tip` lengths which are what usually allow forks to be managed. The only effect a value `> 1` would have here is to increase the time between an Algorand block being seen, and transactions pertaining to it being written.

## :label: Transaction Validation

Blocks submitted to this core form part of the "submission material", which for Algorand comprises the block header, transactions, plus any proofs for any `pToken` relevant transactions. (Proofs via the __[Algorand API v2 RPC call here](https://developer.algorand.org/docs/rest-apis/algod/v2/#get-v2blocksroundtransactionstxidproof)__).

This material - should the block's round number be subsequent - is then stored in the core's database as the `latest block`. As further blocks are submitted, previous blocks eventually become `canon` when they become `canon-to-tip-length` number of blocks away from the latest. Once `canon`, the block's submission material will be examined for merkle-proofs pertaining to `pToken` transactions. Should any proofs be found, the following validation occurs:

 - The transaction to which the proof pertains is examined, checking that it does indeed pertain to a `pToken` ASA transaction.

 - The transaction's ID is then calculated.

 - An Algorand-specific merkle-leaf is constructed using the above transaction ID and the `stib` value provided within the proof.

 - The merkle-proof is then deconstructed into it's constituent hashes, which are then used along with the leaf above to calculate a merkle-root.

 - This root is then compared to the transactions' merkle-root in the block header (The `txn` field). If it is found to be the same, the transaction is considered valid.

Thusly validated, the core will then parse required information from the transaction it in order to write it's own, `pToken` transaction destined for the interim chain.

##
