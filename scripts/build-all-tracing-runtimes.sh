#!/bin/bash

CHAINS=(
    moonbase
    moonriver
)

SRTOOL_IMAGE="paritytech/srtool:1.53.0"

docker pull $SRTOOL_IMAGE

cd tracing

for TRACING_VERSION in *; do
  if [ -d "${TRACING_VERSION}" ]; then
    cd ..
    ./scripts/build-tracing-runtime.sh $TRACING_VERSION
    cd tracing
  fi
done

cd ..
