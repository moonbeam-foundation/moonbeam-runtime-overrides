#!/bin/bash

# Constants
ALL_RUNTIMES_NAMES=(
    moonbase
    moonriver
    moonbeam
)


# Arguments
VERSION=$1
RUNTIME_NAME_FILTER=${2:-"[a-z]"}
CARGO_BUILD_JOBS=${3:-"8"}

# Copy the needed source code in a temporary folder
mkdir -p tmp/build/tracing
cp -r tracing/${VERSION} tmp/build/tracing/${VERSION}
cp -r tracing/shared  tmp/build/tracing/${VERSION}/shared
cd tmp/build/tracing/${VERSION}
chmod -R 777 runtime


# get the toolchain rust version
RUST_VERSION=$(cat rust-toolchain | grep channel | grep --only-matching --perl-regexp "(\d+\.){2}\d+")
# if the version is empty, default to 1.69.0 (that is for runtime before 2300)
if [ -z "$RUST_VERSION" ]; then
  RUST_VERSION="1.69.0"
fi
SRTOOL_IMAGE="paritytech/srtool:$RUST_VERSION"

# Download srtool image
docker pull $SRTOOL_IMAGE

for RUNTIME_NAME in ${ALL_RUNTIMES_NAMES[@]}; do
  RUNTIME_DIR="runtime/$RUNTIME_NAME"
  if [[ "$RUNTIME_NAME" =~ $RUNTIME_NAME_FILTER ]]; then
    if [ -d "$RUNTIME_DIR" ]; then
      echo "Build $RUNTIME_NAME-${VERSION}-substitute-tracing…"

      if [[ "$VERSION" == "local" ]]; then
        cargo build -p $RUNTIME_NAME-runtime --features on-chain-release-build
        cp target/debug/wbuild/$RUNTIME_NAME-runtime/${RUNTIME_NAME}_runtime.wasm ../../../../wasm/$RUNTIME_NAME-runtime-$VERSION-substitute-tracing.wasm
      else
        CMD="docker run \
          -i \
          --rm \
          -e PACKAGE=$RUNTIME_NAME-runtime \
          -e RUNTIME_DIR=$RUNTIME_DIR \
          -e WASM_BUILD_STD=0 \
          -e CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS \
          -e BUILD_OPTS=--features=on-chain-release-build \
          -v $PWD:/build \
          $SRTOOL_IMAGE build --app --json -cM"

        # here we keep streaming the progress and fetch the last line for the json result
        stdbuf -oL $CMD | {
          while IFS= read -r line
          do
            echo ║ $line
            JSON="$line"
          done
          # Copy wasm blob and json digest in git repository
          Z_WASM=`echo $JSON | jq -r .runtimes.compressed.wasm`
          cp $Z_WASM ../../../../wasm/$RUNTIME_NAME-runtime-$VERSION-substitute-tracing.wasm
          echo $JSON > ../../../../srtool-digest/$RUNTIME_NAME-runtime-$VERSION-substitute-tracing.json
        }
      fi


      echo "Finished building $RUNTIME_NAME-$VERSION-substitute-tracing"
    fi
  fi
done

# Move back to git repository root
cd ../../../..
# cleanup
rm -rf tmp
