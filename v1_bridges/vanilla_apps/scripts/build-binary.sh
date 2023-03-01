#!/bin/bash
BINARY_NAME=$1
cd "$(dirname -- $0)"
cd ../
cargo b -r --bin $BINARY_NAME
