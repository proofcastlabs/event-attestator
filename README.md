# Event Attestator

**Note:** this repo is currently a work in progress and it will be update in the incoming days

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


When everything is connected you can manually process a block of interest on the configured chain through the following call:

```bash
curl \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"process","params":["bsc", "38745426", "false", "true"]}' \
  http://127.0.0.1:3030/v1/rpc
```

The output would contain all the signatures for the configured events in the `signed_events` array:

```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "latest_block_num": 38745426,
    "network_id": {
      "chain_id": 56,
      "disambiguator": 0,
      "protocol_id": "Ethereum",
      "version": "V1"
    },
    "signed_events": [
      {
        "block_hash": "0x1535c8e9ab42918c2df9d05ba18a816dafc2e3eba45fffc334d2890a2ebcc0bf",
        "encoded_event": "0100016162636465656666200000000000000000000000000000000000000000000000000000000000000000005af3107a4000",
        "event_id": "abcdeeff",
        "log": {
          "address": "0xf5e11df1ebcf78b6b6d26e04ff19cd786a1e81dc",
          "data": [],
          "topics": [
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
            "0x000000000000000000000000b0116a2cab9774e9d75834dcce64f99fd75a8579",
            "0x0000000000000000000000000000000000000000000000000000000000000000"
          ]
        },
        "public_key": "03572d2b94ea3918ecd104cdd84a07efc1f9cbc1f11bbd02a6418179894d0d2fc6",
        "signature": "02a893f433fcd688558931a1ab8041c8384ac710550e8ab22c705233c68bd12c38c085405ca6d2c5ea3db8d1b6b84d49f46f070239dcc06aa3c4ec78ea72cb0f01",
        "version": "V1"
      }
    ],
    "timestamp": 1716302631
  }
}
```
