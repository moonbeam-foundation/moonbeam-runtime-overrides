#!/bin/bash

CHAINS=(
    moonbase
    moonriver
)

SRTOOL_IMAGE="paritytech/srtool:1.53.0"

docker pull $SRTOOL_IMAGE

VERSION=$1
cd tracing/${VERSION}

# Get copy of shared code and move all dependencies to shared
find . -prune -o  -name '*.toml' -not -path "*/target/*" -exec sed -i 's/..\/shared/shared/g' {} \;
cp -r ../shared shared

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
      Z_WASM=`echo $JSON | jq -r .runtimes.compressed.wasm`
      cp $Z_WASM ../../wasm/$CHAIN-runtime-$VERSION-substitute-tracing.wasm
      echo $JSON > ../../srtool-digest/$CHAIN-runtime-$VERSION-substitute-tracing.json
    }

    echo "Finished building $CHAIN-$VERSION-substitute-tracing"
  fi
done

# Remove copy of shared code and restore all dependencies to shared
rm -rf shared
find . -prune -o  -name '*.toml' -not -path "*/target/*" -exec sed -i 's/..\/shared/..\/..\/shared/g' {} \;

cd ../..
