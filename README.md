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

# Setup

1. Copy the configuration sample into the p
