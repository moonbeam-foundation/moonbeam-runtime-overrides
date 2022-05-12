#!/bin/bash

POLKADOT_VERSION="v0.9.19"

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
  pallets\\/moonbeam-orbiters
  pallets\\/parachain-staking
  pallets\\/proxy-genesis-companion
  pallets\\/xcm-transactor
  precompiles\\/assets-erc20
  precompiles\\/author-mapping
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
)

declare -A SHARED_PATHS
SHARED_PATHS["..\/..\/primitives\/rpc\/evm-tracing-events"]="..\/..\/..\/shared\/primitives\/rpc\/evm-tracing-events"
SHARED_PATHS["..\/rpc\/evm-tracing-events"]="..\/..\/..\/shared\/primitives\/rpc\/evm-tracing-events"
SHARED_PATHS["..\/..\/primitives\/rpc\/debug"]="..\/..\/..\/shared\/primitives\/rpc\/debug"
SHARED_PATHS["..\/evm_tracer"]="..\/..\/..\/shared\/runtime\/evm_tracer"

SPEC_VERSION=$1
GIT_REF=${2:-"runtime-$SPEC_VERSION"}

if [[ "$GIT_REF" == "runtime-$SPEC_VERSION" ]]; then
  GIT_DEP_REF="rev = \"$GIT_REF\""
else
  GIT_DEP_REF="branch = \"$GIT_REF\""
fi

if [[ "$SPEC_VERSION" == "local" ]]; then
  MOONBEAM_PATH="../../"
else
  MOONBEAM_PATH="tmp/moonbeam"

  # Get moonbeam repository snapshot
  echo "Get moonbeam snapshot..."
  rm -rf tmp
  mkdir tmp
  git clone https://github.com/PureStake/moonbeam --depth 1 -b $GIT_REF $MOONBEAM_PATH
fi

# Copy relevant files
echo "Copy relevant files and folders..."
mkdir -p tracing/$SPEC_VERSION/runtime
cp $MOONBEAM_PATH/Cargo.lock tracing/$SPEC_VERSION/Cargo.lock
cp $MOONBEAM_PATH/rust-toolchain tracing/$SPEC_VERSION/rust-toolchain
cp -r $MOONBEAM_PATH/runtime/common tracing/$SPEC_VERSION/runtime/
cp -r $MOONBEAM_PATH/runtime/moon* tracing/$SPEC_VERSION/runtime/

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
  sed -i -e 's/\[\s*"std"\s*\]/\[ "std", "evm-tracing" \]/g' tracing/$SPEC_VERSION/runtime/$CHAIN/Cargo.toml
done

# Replace some path dependencies by git dependencies
echo "Replace path dependencies by git dependencies..."
for PATH_TO_GIT in ${PATHS_TO_GIT[@]}; do
  sed -i -e "s/path = \"..\/..\/$PATH_TO_GIT\"/git = \"https:\/\/github.com\/purestake\/moonbeam\", $GIT_DEP_REF/g" tracing/$SPEC_VERSION/runtime/common/Cargo.toml
  for CHAIN in ${CHAINS[@]}; do
    sed -i -e "s/path = \"..\/..\/$PATH_TO_GIT\"/git = \"https:\/\/github.com\/purestake\/moonbeam\", $GIT_DEP_REF/g" tracing/$SPEC_VERSION/runtime/$CHAIN/Cargo.toml
    sed -i -e "s/path = \"..\/relay-encoder\"/git = \"https:\/\/github.com\/purestake\/moonbeam\", $GIT_DEP_REF/g" tracing/$SPEC_VERSION/runtime/$CHAIN/Cargo.toml
  done
done

cd tracing/$SPEC_VERSION

# Rewrite some path dependencies to use shared dependencies
for K in "${!SHARED_PATHS[@]}"; do
  find . -path './target' -prune -o  -name '*.toml' -exec sed -i "s/path = \"$K\"/path = \"${SHARED_PATHS[$K]}\"/g" {} \;
  echo $K;
done

# Fix cargo.lock
cargo update -p evm

cd ../..
