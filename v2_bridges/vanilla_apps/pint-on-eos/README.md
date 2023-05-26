# :closed_lock_with_key: pINT-on-EOS App

This Provable __pINT-on-EOS__ app leverages the __pToken__ core in order to manages the cross-chain conversions between any __ERC20__ tokens on the ethereum blockchain & their pTokenized equivalents on the __EOS__ blockchain. This app implements a simple CLI and a non-HSM-using database in order to serve as an example for how to consume the core library.

&nbsp;

***

&nbsp;

### :earth_africa: App Overview

#### :lock_with_ink_pen: Security:

This demonstration app includes a simple, _unprotected_, __`rocksDB`__ database to serve as an example for how to implement the database interface required by the core library. As such, sans security-features, this app should be used for test-net and similar only.

Main-net implementations will leverage various __HSMs__ in order to provide either an encrypted database interface as is the case with __Strongbox__, or both that and a fully secure runtime environment as is the case with Intel's __SGX__.

&nbsp;

***

&nbsp;

### :point_right: Usage:

```

```

&nbsp;

***

&nbsp;

###  :page_facing_up: Set Up

Enter the __`/scripts`__ directory & you'll find some sample core initialization scripts plus core initialization JSONs. You can use these to initialize the vanilla version of this `pINT-on-EOS` bridge thusly:

 - First build the binary with __`cargo b --release`__.

 - Once built, initialize the EOS side of the core by running: __`./init-eos.sh`__

 - Then initialize the INT side of the core by running: __`./init-eth.sh`__

 - The INT initialization step will result in an output containing a signed INT transaction over the supplied smart-contract bytecode, which once broadcast will deploy the __`pINT-on-eos`__ pToken contract. Before deploying, you'll first need to fund the __`eth-address`__ which you'll also find in the above output. Once that address is funded with some INT, you may broadcast this transaction.

 - Congratulations, your core is now initialized! You'll know your core is initialized correctly when the command __`<path-to-binary>/perc20_on_eos getEnclaveState`__ returns a JSON containing the core's state.

&nbsp;

***

&nbsp;

### :wrench: Build

You need to ensure you have both __`clang`__ & __`llvm`__ (or later versions) installed on your system. Then enter the __`./app`__ directory and run:

__`❍ cargo build --release`__

To enable __`debug`__ mode in the __`pToken`__ core, do so via the __`Cargo.toml`__ like so:

__`pbtc_core = { path = "<path-to-ptokens-core>", features = ["debug_mode"] }`__

:radioactive: Debug mode __MUST NOT__ be used in production - it bypasses __ALL__ security measures an app may implement, and makes fully accessible the entire database, in plain-text.:radioactive:

#### Versions

 - __`llvm:`__ version 6.0.0 or later.
 - __`clang:`__ version 6.0.0-1ubuntu2 or later.
 - __`rustc & cargo:`__ version 1.42.0-stable or later.

&nbsp;

***

&nbsp;

### :cyclone: Log Rotation

A log for each run of the tool will be written to the __`./logs/`__ directory.

Log rotation occurs when the number of logs reaches the __`MAX_NUM_LOGS`__ threshold. This threshold may be set by the user upon build via an environment variable thusly:

```

MAX_NUM_LOGS=100 cargo b --release

```

__NOTE:__ If no environment variable is provided upon build, the threshold will default to __1000__ logs.

__NOTE:__ The __`MAX_NUM_LOGS`__ also has a lower bound of __20__.

&nbsp;

***

&nbsp;

### :black_nib: Notes

- The maximum __`confs`__ possible during initialization is 255.


&nbsp;

***

&nbsp;

### :guardsman: Tests

To run the tests simply run:

__`❍ cargo test`__

&nbsp;

***

&nbsp;

### :black_nib: To Do:

- [x] INT & EOS debug block reprocessors
