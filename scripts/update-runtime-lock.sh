#!/bin/bash

# Arguments
RUNTIME_TYPE=$1
VERSION=$2

if [[ "$RUNTIME_TYPE" == "tracing" ]]; then
    echo "Copying shared directory in ${RUNTIME_TYPE}/${VERSION}"
    cp -r ${RUNTIME_TYPE}/shared ${RUNTIME_TYPE}/${VERSION}/shared
fi

cd ${RUNTIME_TYPE}/${VERSION}
echo "Running 'cargo fetch' to update the lock file"

echo
cargo fetch
echo 

if [[ "$RUNTIME_TYPE" == "tracing" ]]; then
    echo "Removing shared directory in ${RUNTIME_TYPE}/${VERSION}"
    rm -rf shared
fi

cd ../..
