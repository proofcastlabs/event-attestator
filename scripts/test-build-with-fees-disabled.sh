#!/bin/bash

# Exit early if non-zero exit code encountered
set -e

echo testing fee-disabled core compilations

featureFlag="--features=disable-fees"
packages=(
	"pbtc-on-eos"
	"pbtc-on-eth"
	"peos-on-eth"
	"perc20-on-eos"
	"perc20-on-evm"
)

for package in "${packages[@]}"
do
  echo testing fee-disabled core compilation for package: \'$package\'...
  cargo build --package=$package $featureFlag
done
