#!/bin/bash
set -e
cd "$(dirname -- $0)"

declare -a cores=(
	"pbtc-on-eos"
	"pbtc-on-eth"
	"peos-on-eth"
	"perc20-on-eos"
	"perc20-on-evm"
)

for core in "${cores[@]}"
do
	../$core/run-all-tests.sh
done
