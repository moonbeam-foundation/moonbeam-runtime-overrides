#!/bin/bash

CHAINS=(
    moonbase
    moonriver
)

VERSION=$1

./scripts/build-tracing-runtime.sh $VERSION

CMD="git diff --name-only"

stdbuf -oL $CMD | {
  while IFS= read -r line; do
    echo ║ $line
    for CHAIN in ${CHAINS[@]}; do
      if [[ "$line" == "wasm/$CHAIN-runtime-$VERSION-substitute-tracing.wasm" ]]; then 
        echo "Runtime $CHAIN-$VERSION as changed, please rebuild it locally with command './script/build-tracing-runtime.sh $VERSION'."
        exit 1
      fi
    done
    echo "Runtime $CHAIN-$VERSION is valid."
  done
}