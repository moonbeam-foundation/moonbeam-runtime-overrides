#!/bin/bash

# Constants
ALL_RUNTIMES_NAMES=(
    moonbase
    moonriver
    moonbeam
)
SRTOOL_IMAGE="paritytech/srtool:1.74.0"

# Arguments
VERSION=$1
RUNTIME_NAME_FILTER=${2:-"[a-z]"}
CARGO_BUILD_JOBS=${3:-"8"}

# Download srtool image
docker pull $SRTOOL_IMAGE

# Copy the needed source code in a temporary folder
mkdir -p tmp/build/tracing
cp -r tracing/${VERSION} tmp/build/tracing/${VERSION}
cp -r tracing/shared  tmp/build/tracing/${VERSION}/shared
cd tmp/build/tracing/${VERSION}
chmod -R 777 runtime

for RUNTIME_NAME in ${ALL_RUNTIMES_NAMES[@]}; do
  RUNTIME_DIR="runtime/$RUNTIME_NAME"
  if [[ "$RUNTIME_NAME" =~ $RUNTIME_NAME_FILTER ]]; then
    if [ -d "$RUNTIME_DIR" ]; then
      echo "Build $RUNTIME_NAME-${VERSION}-substitute-tracing…"

      if [[ "$VERSION" == "local" ]]; then
        cargo build -p $RUNTIME_NAME-runtime
        cp target/debug/wbuild/$RUNTIME_NAME-runtime/${RUNTIME_NAME}_runtime.wasm ../../../../wasm/$RUNTIME_NAME-runtime-$VERSION-substitute-tracing.wasm
      else
        CMD="docker run \
          -i \
          --rm \
          -e PACKAGE=$RUNTIME_NAME-runtime \
          -e RUNTIME_DIR=$RUNTIME_DIR \
          -e CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS \
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
