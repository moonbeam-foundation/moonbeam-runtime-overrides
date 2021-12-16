#!/bin/bash

# Constants
CHAINS=(
    moonbase
    moonriver
    moonbeam
)
SRTOOL_IMAGE="paritytech/srtool:1.53.0"

# Arguments
VERSION=$1

# Download srtool image
docker pull $SRTOOL_IMAGE

# Copy the needed source code in a temporary folder
mkdir -p tmp/build/tracing
cp -r tracing/${VERSION} tmp/build/tracing/${VERSION}
cp -r tracing/shared  tmp/build/tracing/${VERSION}/shared
cd tmp/build/tracing/${VERSION}

# Move all dependencies to shared (to be in the rust workspace)
find . -path './target' -prune -o  -name '*.toml' -exec sed -i 's/..\/..\/shared/..\/shared/g' {} \;

for CHAIN in ${CHAINS[@]}; do
  RUNTIME_DIR="runtime/$CHAIN"
  if [ -d "$RUNTIME_DIR" ]; then
    echo "Build $CHAIN-${VERSION}-substitute-tracing…"

    CMD="docker run -i --rm -e PACKAGE=$CHAIN-runtime -e RUNTIME_DIR=$RUNTIME_DIR -v $PWD:/build $SRTOOL_IMAGE build --app --json -cM"

    # here we keep streaming the progress and fetch the last line for the json result
    stdbuf -oL $CMD | {
      while IFS= read -r line
      do
        echo ║ $line
        JSON="$line"
      done
      # Copy wasm blob and josn digest in git repository
      Z_WASM=`echo $JSON | jq -r .runtimes.compressed.wasm`
      cp $Z_WASM ../../../../wasm/$CHAIN-runtime-$VERSION-substitute-tracing.wasm
      echo $JSON > ../../../../srtool-digest/$CHAIN-runtime-$VERSION-substitute-tracing.json
    }

    echo "Finished building $CHAIN-$VERSION-substitute-tracing"
  fi
done

# Move back to git repository root
cd ../../../..
