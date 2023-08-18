#!/bin/bash

RUNTIME_TYPE=$1
SPEC_VERSION=$2
GIT_REF=${3:-"runtime-$SPEC_VERSION"}

if [[ "$GIT_REF" == "runtime-$SPEC_VERSION" ]]; then
  GIT_DEP_REF="--rev $GIT_REF"
else
  GIT_DEP_REF="--branch $GIT_REF"
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
mkdir -p $RUNTIME_TYPE/$SPEC_VERSION/runtime
cp $MOONBEAM_PATH/Cargo.toml $RUNTIME_TYPE/$SPEC_VERSION/Cargo.toml
cp $MOONBEAM_PATH/Cargo.lock $RUNTIME_TYPE/$SPEC_VERSION/Cargo.lock
cp $MOONBEAM_PATH/rust-toolchain $RUNTIME_TYPE/$SPEC_VERSION/rust-toolchain
cp -r $MOONBEAM_PATH/runtime/common $RUNTIME_TYPE/$SPEC_VERSION/runtime/
cp -r $MOONBEAM_PATH/runtime/moon* $RUNTIME_TYPE/$SPEC_VERSION/runtime/

# Remove irrelevant files
if [[ "$RUNTIME_TYPE" == "tracing" ]]; then
  rm -rf $RUNTIME_TYPE/$SPEC_VERSION/runtime/relay-encoder
fi

echo "Run migration script"
cd scripts
cargo run -q --bin migrate-imported -- --dir ../$RUNTIME_TYPE/$SPEC_VERSION --repo "https://github.com/PureStake/moonbeam" --runtime-type $RUNTIME_TYPE $GIT_DEP_REF
cd ..

echo "Running ./scripts/update-runtime-lock.sh $SPEC_VERSION ..."
source ./scripts/update-runtime-lock.sh $SPEC_VERSION

echo "Done !"
