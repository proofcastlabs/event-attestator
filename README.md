# Event Attestator

This repository contains the core codebase for the Proofcast event attestation
system.
This will be useful for anyone aiming at proving that an event or a state change
happened on a blockchain to another one, backed by a Trusted Execution Environment
attestation.

Currently the TEE supported is Android Strongbox, but other platforms will be
integrated soon.

The project is structured like follows:

 - **`common/`**: low level library providing all the code for block
 validation and proof generation for each supported blockchain
 - **`apps/`**: lightweight JSONRPC HTTP server abstracting the interaction to
 the TEE contained into the device


# Architecture

At a glance, the system architecture is better understood from the following
diagram:

![diagram-1](/docs/imgs/authorization-system.png)


 - The JSONRPC api allows the end user to send commands to the device
 - These commands are then sent through the websocket channel to which the device is connects on startup
 - The core library deserialize each message coming from the websocket channel and process them accordingly

# Setup for EVM chains

1. Copy the configuration sample from `sample-config.toml` file in the common folder
and configure it to your needs, in particular be careful to fill up the ```[networks.chainName]``` section with the `endpoints` property.

1. For each configured chain, set the `events` array where each element is in the following form:

```
["<contract-address>","<topic>"]
```

3. Build the app

```
cargo build --release --bin jsonrpc-app
```

4. Run

```
./jsonrpc-app -c <config.toml>
```


5. Then follow the README on [this repo](https://github.com/proofcastlabs/tee-wrapper-android) in order to connect app to the device.
