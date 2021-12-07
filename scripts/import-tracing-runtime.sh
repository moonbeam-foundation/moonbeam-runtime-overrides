#!/bin/bash

POLKADOT_VERSION="v0.9.12"

CHAINS=(
  moonbase
  moonriver
  moonbeam
)

PATHS_TO_GIT=(
  core-primitives
  pallets\\/asset-manager
  pallets\\/author-mapping
  pallets\\/ethereum-chain-id
  pallets\\/maintenance-mode
  pallets\\/migrations
  pallets\\/parachain-staking
  pallets\\/proxy-genesis-companion
  pallets\\/xcm-transactor
  precompiles\\/assets-erc20
  precompiles\\/balances-erc20
  precompiles\\/crowdloan-rewards
  precompiles\\/parachain-staking
  precompiles\\/pallet-democracy
  precompiles\\/relay-encoder
  precompiles\\/utils
  precompiles\\/xcm_transactor
  precompiles\\/xtokens
  primitives\\/account\\/
  primitives\\/rpc\\/txpool
  primitives\\/xcm\\/
  relay-encoder
)

declare -A SHARED_PATHS
SHARED_PATHS["..\/..\/primitives\/rpc\/evm-tracing-events"]="..\/..\/..\/shared\/primitives\/rpc\/evm-tracing-events"
SHARED_PATHS["..\/rpc\/evm-tracing-events"]="..\/..\/..\/shared\/primitives\/rpc\/evm-tracing-events"

SPEC_VERSION=$1
GIT_REF=${2:-"runtime-$SPEC_VERSION"}

# Get moonbeam repository shopshot
echo "Get moonbeam branch..."
rm -rf tmp
mkdir tmp
git clone https://github.com/PureStake/moonbeam -b $GIT_REF tmp/moonbeam

# Copy relevant files
echo "Copy relevant files and folders..."
mkdir -p tracing/$SPEC_VERSION/primitives/rpc
cp tmp/moonbeam/Cargo.lock tracing/$SPEC_VERSION/Cargo.lock
cp tmp/moonbeam/rust-toolchain tracing/$SPEC_VERSION/rust-toolchain
cp -r tmp/moonbeam/primitives/ext tracing/$SPEC_VERSION/primitives
cp -r tmp/moonbeam/primitives/rpc/debug tracing/$SPEC_VERSION/primitives/rpc
cp -r tmp/moonbeam/runtime tracing/$SPEC_VERSION

# Remove irrelevant files
rm -rf tracing/$SPEC_VERSION/runtime/relay-encoder

# Create a virtual manifest for this new rust workspace from tracing template
echo "Create a virtual manifest for this new rust workspace"
cp tracing/Cargo.toml.template tracing/$SPEC_VERSION/Cargo.toml

# Set evm branch
sed -i -e "s/POLKADOT_VERSION/$POLKADOT_VERSION/g" tracing/$SPEC_VERSION/Cargo.toml

# Enable evm-tracing feature
echo "Enable evm-tracing feature..."
for CHAIN in ${CHAINS[@]}; do
  sed -i -e 's/\["std"\]/\["std", "evm-tracing"\]/g' tracing/$SPEC_VERSION/runtime/$CHAIN/Cargo.toml
done

# Replace some path dependencies by git dependencies
echo "Replace path dependencies by git dependencies..."
for PATH_TO_GIT in ${PATHS_TO_GIT[@]}; do
  sed -i -e "s/path = \"..\/..\/$PATH_TO_GIT\"/git = \"https:\/\/github.com\/purestake\/moonbeam\", rev = \"runtime-$SPEC_VERSION\"/g" tracing/$SPEC_VERSION/runtime/common/Cargo.toml
  for CHAIN in ${CHAINS[@]}; do
    sed -i -e "s/path = \"..\/..\/$PATH_TO_GIT\"/git = \"https:\/\/github.com\/purestake\/moonbeam\", rev = \"runtime-$SPEC_VERSION\"/g" tracing/$SPEC_VERSION/runtime/$CHAIN/Cargo.toml
  done
done

# Rewrite some path dependencies to use shared dependencies
cd tracing/$SPEC_VERSION
for K in "${!SHARED_PATHS[@]}"; do
  find . -path './target' -prune -o  -name '*.toml' -exec sed -i "s/path = \"$K\"/path = \"${SHARED_PATHS[$K]}\"/g" {} \;
  echo $K;
done
cd ../..
