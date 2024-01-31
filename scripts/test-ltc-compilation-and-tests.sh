#/bin/bash

# NOTE: Exit early if non zero exit code is encountered
set -e

echo testing litecoin core compilations

featureFlag="--features=ltc"
packages=("pbtc-on-int")

for package in "${packages[@]}"
do
  echo testing litecoin compilation for package: \'$package\'...
  cargo test --package=$package $featureFlag
done
