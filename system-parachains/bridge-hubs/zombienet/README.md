# Bridges Tests for Local Polkadot <> Kusama Bridge

This folder contains [zombienet](https://github.com/paritytech/zombienet/) based integration tests for both
onchain and offchain bridges code. Due to some
[technical diffuculties](https://github.com/paritytech/parity-bridges-common/pull/2649#issue-1965339051), we
are using native zombienet provider, which means that you need to build some binaries locally.

To start those tests, you need to:

- download latest [zombienet release](https://github.com/paritytech/zombienet/releases);

- build Polkadot binary by running `cargo build -p polkadot --release` command in the
[`polkadot-sdk`](https://github.com/paritytech/polkadot-sdk) repository clone. Also
`cargo build -p polkadot-prepare-worker --release`
`cargo build -p polkadot-execute-worker --release`;

- build Polkadot Parachain binary by running `cargo build -p polkadot-parachain-bin --release` command in the
[`polkadot-sdk`](https://github.com/paritytech/polkadot-sdk) repository clone;

- ensure that you have [`node`](https://nodejs.org/en) installed. Additionally, we'll need globally installed
`polkadot/api-cli` package (use `npm install -g @polkadot/api-cli@beta` to install it);

- build Substrate relay by running `cargo build -p substrate-relay --release` command in the
[`parity-bridges-common`](https://github.com/paritytech/parity-bridges-common) repository clone;


???

# zombienet: v1.3.70 (v1.3.78 not working)
	# TODO: add branch and or fix
# polkadot-sdk (polkadot + polkadot-parachain): v1.3.0-rc1
# polkadot-sdk (Kusama and Polkadot chain specs): 122086d3d5
# polkadot-sdk (chain-spec-builder): cargo build --release -p staging-chain-spec-builder

sudofi


cd polkadot-sdk
git checkout 122086d3d5
cd polkadot
cd polkadot-sdk
cargo run -p polkadot -- build-spec --chain=polkadot-local >../runtimes/system-parachains/bridge-hubs/zombienet/networks/polkadot-local.json
cargo run -p polkadot -- build-spec --chain=kusama-local >../runtimes/system-parachains/bridge-hubs/zombienet/networks/kusama-local.json

cargo run -p polkadot build-spec --chain=westend-local >../runtimes/system-parachains/bridge-hubs/zombienet/networks/westend-local.json



- build all involved runtimes from this repo:
`srtool build -p staging-kusama-runtime -r relay/kusama --root --build-opts=--features=fast-runtime --verbose`
`srtool build -p polkadot-runtime -r relay/polkadot --root --build-opts=--features=fast-runtime --verbose`
`srtool build -p asset-hub-kusama-runtime -r system-parachains/asset-hubs/asset-hub-kusama --root`
`srtool build -p asset-hub-polkadot-runtime -r system-parachains/asset-hubs/asset-hub-polkadot --root`
`srtool build -p bridge-hub-kusama-runtime -r system-parachains/bridge-hubs/bridge-hub-kusama --root`
`srtool build -p bridge-hub-polkadot-runtime -r system-parachains/bridge-hubs/bridge-hub-polkadot --root`

- copy fresh `substrate-relay` binary, built in previous point, to the `~/local_bridge_testing/bin/substrate-relay`;

- change the `POLKADOT_SDK_FOLDER` and `ZOMBIENET_BINARY_PATH` (and ensure that the nearby variables
have correct values) in the `./run-tests.sh`.

After that, you could run tests with the `./run-tests.sh` command. Hopefully, it'll show the
"All tests have completed successfully" message in the end. Otherwise, it'll print paths to zombienet
process logs, which, in turn, may be used to track locations of all spinned relay and parachain nodes.


Benchmarks:
cargo build -p polkadot-parachain-bin --release --features runtime-benchmarks
srtool build -p bridge-hub-polkadot-runtime -r system-parachains/bridge-hubs/bridge-hub-polkadot --root --build-opts=--features=runtime-benchmarks
(dance with chain spec - merge code from freshly built runtime and other stuff from a regular chain spec, built using zombienet tests)
../polkadot-sdk/target/release/polkadot-parachain-benchmarks benchmark pallet --chain=/tmp/zombie-2d0df747d18ce0425f92983897ebfb79_-1890719-uGtl8quFPDkx/1002-rococo-local.json --pallet=pallet_bridge_messages --extrinsic=*
