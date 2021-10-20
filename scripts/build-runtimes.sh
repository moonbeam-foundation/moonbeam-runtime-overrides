#!/bin/bash

CHAINS=(
    moonbase
    moonriver
)

SRTOOL_IMAGE="paritytech/srtool:1.53.0"

docker pull $SRTOOL_IMAGE
cd tracing

for VERSION in *; do
  if [ -d "${VERSION}" ]; then
      cd ${VERSION}
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
      cd ..
  fi
done
