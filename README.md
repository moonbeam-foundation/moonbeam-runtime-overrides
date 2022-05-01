# Moonbeam Runtime Overrides

This repository contains rewrites of the moonbeam project runtimes, these rewrites allow some specialized nodes to run modified runtimes, for example for tracing purposes.

## Tracing

The moonbeam tracing nodes allow to trace the execution of ethereum transactions via specific RPC requests. This requires special features on the runtime side, which are only present in runtimes modified for tracing, called "tracing runtime".

The tracing runtimes code is in the `tracing` folder of this git repository. All past spec versions of the runtime are rewritten, each in the subfolder `tracing/XXX` where `XXX` is the spec version number.

Some parts of the code are common to all versions of the tracing runtime, they are in the `tracing/shared` folder. This is possible because past runtimes can be rewritten, as long as they respect the API exposed by the on-chain runtime of the same version.

### Import a new tracing runtime

Each time the moonbeam project delivers a new runtime, the associated tracing runtime must be created and deployed on the tracing nodes before the on-chain runtime upgrade.

Below is the procedure to create a new tracing runtime from a new version of the moonbeam runtimes.
Note that we speak of runtimes in the plural, because there are 3 different runtimes for our 3 public networks:

- `moonbase-runtime` for network "Moonbase Alpha" (test network)
- `moonriver-runtime` for network "Moonriver" (parachain on Kusama)
- `moonbeam-runtime` for network "Moonbeam" (parachain on Polkadot)

So, to import the new runtimes with chain spec `XXX`:

1. Clone this git repository and create a branch `tracing-runtime-XXX` based on `main`
1. Run the following command in the root of this git repository: `./scripts/import-tracing-runtime.sh XXX`
1. Move to folder `tracing/XXX`
1. Run command `cargo update - p evm`
1. Commit new runtimes code
1. Go back to the root of this git repository
1. Run command `./scripts/build-tracing-runtime.sh XXX`
1. Commit wasm blob and json digest for each new runtime
1. Push the branch `tracing-runtime-XXX` and submit a PR
