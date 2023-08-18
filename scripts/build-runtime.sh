#!/bin/bash

# Constants
ALL_RUNTIMES_NAMES=(
    moonbase
    moonriver
    moonbeam
)
SRTOOL_IMAGE="paritytech/srtool:1.66.1"

# Arguments
RUNTIME_TYPE=$1
VERSION=$2
RUNTIME_NAME_FILTER=${2:-"[a-z]"}
CARGO_BUILD_JOBS=${3:-"8"}

# Download srtool image
docker pull $SRTOOL_IMAGE

# Copy the needed source code in a temporary folder
mkdir -p tmp/build/${RUNTIME_TYPE}
cp -r ${RUNTIME_TYPE}/${VERSION} tmp/build/${RUNTIME_TYPE}/${VERSION}

## Copy the shared folder, if any
if [ -d "$RUNTIME_TYPE/shared" ]; then
  cp -r ${RUNTIME_TYPE}/shared tmp/build/${RUNTIME_TYPE}/${VERSION}/shared
fi

# Go to the temporary folder and allow access to runtime subfolder (needed for docker)
cd tmp/build/${RUNTIME_TYPE}/${VERSION}
chmod -R 777 runtime

for RUNTIME_NAME in ${ALL_RUNTIMES_NAMES[@]}; do
  RUNTIME_DIR="runtime/$RUNTIME_NAME"
  if [[ "$RUNTIME_NAME" =~ $RUNTIME_NAME_FILTER ]]; then
    if [ -d "$RUNTIME_DIR" ]; then
      echo "Build $RUNTIME_NAME-${VERSION}-substitute-${RUNTIME_TYPE}…"

      if [[ "$VERSION" == "local" ]]; then
        cargo build -p $RUNTIME_NAME-runtime
        cp target/debug/wbuild/$RUNTIME_NAME-runtime/${RUNTIME_NAME}_runtime.compact.compressed.wasm ../../../../wasm/$RUNTIME_NAME-runtime-$VERSION-substitute-${RUNTIME_TYPE}.wasm
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
          cp $Z_WASM ../../../../wasm/$RUNTIME_NAME-runtime-$VERSION-substitute-${RUNTIME_TYPE}.wasm
          echo $JSON > ../../../../srtool-digest/$RUNTIME_NAME-runtime-$VERSION-substitute-${RUNTIME_TYPE}.json
        }
      fi


      echo "Finished building $RUNTIME_NAME-$VERSION-substitute-${RUNTIME_TYPE}"
    fi
  fi
done

# Move back to git repository root
cd ../../../..
