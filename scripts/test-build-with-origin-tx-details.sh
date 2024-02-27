#!/bin/bash

# Exit early if non-zero exit code encountered
set -e

echo testing cores with origin chain tx pass through enabled

featureFlag="--features=include-origin-tx-details"

packages=(
	"int_on_evm"
	"erc20_on_int"
)

for package in "${packages[@]}"
do
  echo testing cores with origin chain tx pass through enabled for package: \'$package\'...
  cargo test --package=$package $featureFlag
done
