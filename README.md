# :closed_lock_with_key: pTokens Core

The Provable __pTokens__ core which manages the cross-chain conversions between a host and a native blockchain.

&nbsp;

## :earth_africa: Core Overview

The __pToken__ core is a library implementing light-clients for various block-chains. The initial release involves __ETH__ as the host chain on which the __pTokens__ are manifest, and uses __BTC__ as the native chain and underlying asset.

The core library has zero network connectivity and makes no network requests. It is a push-only model, requiring external tools to gather & feed it the blocks from the chains with which the core is to interact.

In order to initialize the light-clients inside the core, an initial block from each desired chain is required. These will be the only trusted blocks in the system. Thereafter, subsequent blocks pushed to the core will undergo all the usual validation checks w/r/t to that block's veracity before appending it to the small piece of chain the light client holds.

The length of these small pieces of chain held by the core is governed by its __`canon-to-tip`__ length, which length can also be thought of as the number of __`confirmations + 1`__ required before the core will sign a transaction.

Once a block reaches __`canon-to-tip`__ number of blocks away from the tip of the chain, it becomes the __`canon-block`__. At this point, it is searched for any relevant deposit or redemption events and any required transactions are then signed and returned from the core in __`JSON`__ format.

In order to keep the light-clients thin, blocks behind the __`canon-block`__  are removed. In order to do that whilst retaining the integrity of the chain, the block to be removed is first _linked_ to the initial trusted block (the __`anchor-block`__) by hashing it together with the so-called __`linker-hash`__ (where an arbitrary constant is used for the first linkage) and the block to be removed. This way the small piece of chain inside then core can always be proven to have originated from the original trusted block.

And so thusly the core remains synced with the each blockchain, writing relevant transactions as it does so.

## :lock_with_ink_pen: Security:

The library herein is designed to be imported by an application that leverages an HSM in order to implement a secure database that adheres to the interface as defined in __`./src/traits.rs`__.

This library itself implements no such protections, except those afforded by the protected runtime of an __SGX__ environment if an __`app`__ were to leverage such technology.

Note the library can be built in __`debug`__ mode via setting the feature flag when building thusly: __`cargo build --release --features=debug`__.  If built in __`debug`__ mode, all security of the app/core combination are waived entirely, the database is entirely accessible including the private keys!

#### :warning: The core should NOT be used in `debug` mode in production! :warning:

&nbsp;

## :wrench: Build

You need to ensure you have both __`clang`__ & __`llvm`__ (or later versions) installed on your system. Then enter the __`./app`__ directory and run:

__`❍ cargo build --release`__

#### Versions

 - __`llvm:`__ version 6.0.0 or later.
 - __`clang:`__ version 6.0.0-1ubuntu2 or later.
 - __`rustc & cargo:`__ version 1.56.0 or later.

&nbsp;

## :floppy_disk: Database Interface

The `core` implements a generic database whose interface follows:

```
pub trait DatabaseInterface {
    fn end_transaction(&self) -> Result<()>;
    fn start_transaction(&self) -> Result<()>;
    fn delete(&self, key: Bytes) -> Result<()>;
    fn get(&self, key: Bytes, data_sensitivity: Option<u8>) -> Result<Bytes>;
    fn put(&self, key: Bytes, value: Bytes, data_sensitivity: Option<u8>) -> Result<()>;
}

```

The `start_transaction` and `end_transaction` are used by the core algorithms to signal when databasing actions begin and end, allowing a consumer of the `core` to implement atomic databasing however they wish.

Further, the `sensitivity` parameter provides a way for the `core` to signal to the consumer how sensitive the data being transmitted is, giving flexibility for the `core` consumer to handle different levels of sensitive data in different ways, where `0` signifies the _least_ sensitive data, and `255` the _most_.

&nbsp;

## :label: Metadata Chain IDs

The `v2` of this core use metadata chain IDs to route peg-ins and peg-outs to their correct destinations. The byte encodings of those metadata chain IDs are as follows:

```

EthUnknown: 0x00000000
BtcUnknown: 0x01000000
EosUnknown: 0x02000000
Eos Mainnet: 0x02e7261c
FIO Mainnet: 0x02174f20
xDai Mainnet: 0x00f1918e
Ultra Mainnet: 0x025d3c68
Ultra Testnet: 0x02b5a4d6
Telos Mainnet: 0x028c7109
Interim Chain: 0xffffffff
Fantom Mainnet: 0x0022af98
Bitcoin Mainnet: 0x01ec97de
Polygon Mainnet: 0x0075dd4c
Bitcoin Testnet: 0x018afeb2
Ethereum Mainnet: 0x005fe7f9
Ethereum Ropsten: 0x0069c322
Ethereum Rinkeby: 0x00f34368
Arbitrum Mainnet: 0x00ce98c4
Luxochain Mainnet: 0x00d5beb0
EOS Jungle Testnet: 0x0282317f
Binance Chain Mainnet: 0x00e4b170

```

&nbsp;

## :black_nib: Notes

- The maximum __`confs`__ possible during initialization is 255.

- There are hardcoded "safe" addresses for each chain which are used as destinations for transactions whose actual destinations are absent or malformed when being parsed from their originating transactions.

- When initializing the core, the merkle-roots for transactions in blocks are __NOT__ verified - only the block headers are checked. For smaller initialiazation material, feel free to provide empty arrays for the transactions. Ensure not relevant transactions took place in the blocks used to initialize the core.

- The light __BTC__ client implemented herein currently accepts only _two_ deposit types:

1) `p2sh` deposits made to addresses generated via the __`deposit-address-generator`__ run with the private-key emitted by the core upon BTC initialization.
2) `P2PKH` deposits that include in the transaction a UTXO to the `p2pkh` of the aforementioned private-key.

:warning: Neither `p2pk` nor `segwit` transactions are currently supported. Deposits made via such transactions will result in lost funds! :warning:

- The library follows semantic versioning specification ([SemVer](https://semver.org)).

&nbsp;

## :mag: Features

When importing this core library into your app, enable features in your __`Cargo.toml`__ like so:

__`pbtc_core = { version = "0.1.0", features = ["btc-on-eth"] }`__.

Currently supported features include:

 - __`debug`__ To enable debug mode.

 - __`btc-on-eth`__ For the pBTC, BTC on ETH implementation.


&nbsp;

## :guardsman: Tests

To run the tests simply run:

__`❍ cargo test --features='<chosen-feature>'`__

&nbsp;

## :black_nib: To Do:

- [ ] Needs method to adjust difficulty in future.
