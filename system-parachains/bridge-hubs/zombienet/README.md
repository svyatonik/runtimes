# Bridges Tests for Local Polkadot <> Kusama Bridge

This folder contains [zombienet](https://github.com/paritytech/zombienet/) based integration tests for both
onchain and offchain bridges code. Due to some
[technical diffuculties](https://github.com/paritytech/parity-bridges-common/pull/2649#issue-1965339051), we
are using native zombienet provider, which means that you need to build some binaries locally.

To start those tests, you need to:

- until tests are changed to support yet unmerged `chain-spec-generator`, you'll need a patched version of zombienet
  1.3.70 and a small fix. So :
  ```bash
  npm install -g pkg
  git clone https://github.com/svyatonik/zombienet
  cd zombienet/javascript
  git checkout v.1.3.70-with-fix
  npm install
  npm run build
  cd packages/cli
  pkg . -o ../../bins/zombienet-linux -t node18-linux-x64,node18-linux-arm64
  ```
  ;

- build Polkadot binary by running `cargo build -p polkadot --release` command in the
[`polkadot-sdk`](https://github.com/paritytech/polkadot-sdk) repository clone;

- build Polkadot Parachain binary by running `cargo build -p polkadot-parachain-bin --release` command in the
[`polkadot-sdk`](https://github.com/paritytech/polkadot-sdk) repository clone;

- ensure that you have [`node`](https://nodejs.org/en) installed. Additionally, we'll need globally installed
`polkadot/api-cli` package (use `npm install -g @polkadot/api-cli@beta` to install it);

- build Substrate relay by running `cargo build -p substrate-relay --release` command in the
[`parity-bridges-common`](https://github.com/paritytech/parity-bridges-common) repository clone;

- copy fresh `substrate-relay` binary, built in previous point, to the `~/local_bridge_testing/bin/substrate-relay`;

- build all involved runtimes from this repo:
`srtool build -p staging-kusama-runtime -r relay/kusama --root --build-opts=--features=fast-runtime --verbose`
`srtool build -p polkadot-runtime -r relay/polkadot --root --build-opts=--features=fast-runtime --verbose`
`srtool build -p asset-hub-kusama-runtime -r system-parachains/asset-hubs/asset-hub-kusama --root`
`srtool build -p asset-hub-polkadot-runtime -r system-parachains/asset-hubs/asset-hub-polkadot --root`
`srtool build -p bridge-hub-kusama-runtime -r system-parachains/bridge-hubs/bridge-hub-kusama --root`
`srtool build -p bridge-hub-polkadot-runtime -r system-parachains/bridge-hubs/bridge-hub-polkadot --root`

- change the `POLKADOT_SDK_FOLDER` and `ZOMBIENET_BINARY_PATH` (and ensure that the nearby variables
have correct values) in the `./run-tests.sh`.
