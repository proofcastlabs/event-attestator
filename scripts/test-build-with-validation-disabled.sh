#!/bin/bash

# Exit early if non-zero exit code encountered
set -e

echo testing non-validating core compilations

featureFlag="--features=non-validating"

packages=(
	"pbtc-on-eos"
	"pbtc-on-eth"
	"peos-on-eth"
	"perc20-on-eos"
	"perc20-on-evm"
	"pbtc-on-int"
	"peos-on-int"
	"perc20-on-int"
	"pint-on-algo"
	"pint-on-eos"
	"pint-on-evm"
)

for package in "${packages[@]}"
do
  echo testing non-validating core compilation for package: \'$package\'...
  cargo build --package=$package $featureFlag
done
