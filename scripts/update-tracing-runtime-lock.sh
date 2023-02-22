#!/bin/bash

# Arguments
VERSION=$1

echo "Copying shared directory in tracing/${VERSION}"
cp -r tracing/shared tracing/${VERSION}/shared

cd tracing/${VERSION}
echo "Running `cargo fetch` to update the lock file"

echo
cargo fetch
echo 

echo "Removing shared directory in tracing/${VERSION}"
rm -rf shared

cd ../..