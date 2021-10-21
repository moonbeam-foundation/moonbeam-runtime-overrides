#!/bin/bash

CHAINS=(
    moonbase
    moonriver
)

SRTOOL_IMAGE="paritytech/srtool:1.53.0"

docker pull $SRTOOL_IMAGE

for TRACING_VERSION in tracing/*; do
  if [ -d "${TRACING_VERSION}" ]; then
    ./script/build-runtime.sh $TRACING_VERSION
  fi
done
