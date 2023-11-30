#!/bin/bash
#set -eu
shopt -s nullglob

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

# assuming that we'll be using native provide && all processes will be executing locally
# (we need absolute paths here, because they're used when scripts are called by zombienet from tmp folders)
export SCRIPT_FOLDER=`realpath $(dirname "$0")`
export BRIDGE_TESTS_FOLDER=$SCRIPT_FOLDER/tests
export FELLOWSHIP_FOLDER=$SCRIPT_FOLDER/../../..
export POLKADOT_SDK_FOLDER=$SCRIPT_FOLDER/../../../../polkadot-sdk

export POLKADOT_BINARY_PATH=$POLKADOT_SDK_FOLDER/target/release/polkadot
export POLKADOT_PARACHAIN_BINARY_PATH=$POLKADOT_SDK_FOLDER/target/release/polkadot-parachain
export CHAIN_SPEC_BUILDER_PATH=$POLKADOT_SDK_FOLDER/target/release/chain-spec-builder
#export ZOMBIENET_BINARY_PATH=~/local_bridge_testing/bin/zombienet-linux-x64-v.1.3.70
export ZOMBIENET_BINARY_PATH=/home/svyatonik/dev/zombienet/javascript/bins/zombienet-linux-x64

export KUSAMA_WASM_BLOB=$FELLOWSHIP_FOLDER/relay/kusama/target/srtool/release/wbuild/staging-kusama-runtime/staging_kusama_runtime.compact.compressed.wasm
export KUSAMA_ASSET_HUB_WASM_BLOB=$FELLOWSHIP_FOLDER/system-parachains/asset-hubs/asset-hub-kusama/target/srtool/release/wbuild/asset-hub-kusama-runtime/asset_hub_kusama_runtime.compact.compressed.wasm
export KUSAMA_BRIDGE_HUB_WASM_BLOB=$FELLOWSHIP_FOLDER/system-parachains/bridge-hubs/bridge-hub-kusama/target/srtool/release/wbuild/bridge-hub-kusama-runtime/bridge_hub_kusama_runtime.compact.compressed.wasm

export POLKADOT_WASM_BLOB=$FELLOWSHIP_FOLDER/relay/polkadot/target/srtool/release/wbuild/polkadot-runtime/polkadot_runtime.compact.compressed.wasm
export POLKADOT_ASSET_HUB_WASM_BLOB=$FELLOWSHIP_FOLDER/system-parachains/asset-hubs/asset-hub-polkadot/target/srtool/release/wbuild/asset-hub-polkadot-runtime/asset_hub_polkadot_runtime.compact.compressed.wasm
export POLKADOT_BRIDGE_HUB_WASM_BLOB=$FELLOWSHIP_FOLDER/system-parachains/bridge-hubs/bridge-hub-polkadot/target/srtool/release/wbuild/bridge-hub-polkadot-runtime/bridge_hub_polkadot_runtime.compact.compressed.wasm

# bridge configuration
export LANE_ID="00000001"

# tests configuration
ALL_TESTS_FOLDER=`mktemp -d`

function start_coproc() {
    local command=$1
    local name=$2
    local coproc_log=`mktemp -p $TEST_FOLDER`
    coproc COPROC {
        $command >$coproc_log 2>&1
    }
    TEST_COPROCS[$COPROC_PID, 0]=$name
    TEST_COPROCS[$COPROC_PID, 1]=$coproc_log
    echo "Spawned $name coprocess. StdOut + StdErr: $coproc_log"

    return $COPROC_PID
}

# prepare chain specifications
# ../polkadot-sdk/target/release/chain-spec-builder --chain-spec-path test.json runtime -r ./system-parachains/bridge-hubs/bridge-hub-polkadot/target/srtool/release/wbuild/bridge-hub-polkadot-runtime/bridge_hub_polkadot_runtime.compact.compressed.wasm default
# ../polkadot-sdk/target/release/chain-spec-builder --chain-spec-path test-raw.json runtime -s -r ./system-parachains/bridge-hubs/bridge-hub-polkadot/target/srtool/release/wbuild/bridge-hub-polkadot-runtime/bridge_hub_polkadot_runtime.compact.compressed.wasm default
# ../polkadot-sdk/target/release/polkadot-parachain build-spec --chain ./system-parachains/bridge-hubs/zombienet/networks/rococo-bridge-hub-local.json --raw

# ../polkadot-sdk/target/release/chain-spec-builder --chain-spec-path test-raw.json runtime -s -r ./system-parachains/bridge-hubs/bridge-hub-polkadot/target/srtool/release/wbuild/bridge-hub-polkadot-runtime/bridge_hub_polkadot_runtime.compact.compressed.wasm patch --patch-path ./system-parachains/bridge-hubs/zombienet/networks/rococo-bridge-hub-local.json
# ../polkadot-sdk/target/release/polkadot-parachain export-genesis-state --chain test-raw.json
echo >/tmp/my
echo >/tmp/my1


# execute every test from tests folder
TEST_INDEX=1
while true
do
    declare -A TEST_COPROCS
    TEST_COPROCS_COUNT=0
    TEST_PREFIX=$(printf "%04d" $TEST_INDEX)

    # it'll be used by the `sync-exit.sh` script
    export TEST_FOLDER=`mktemp -d -p $ALL_TESTS_FOLDER`

    # check if there are no more tests
    zndsl_files=($BRIDGE_TESTS_FOLDER/$TEST_PREFIX-*.zndsl)
    if [ ${#zndsl_files[@]} -eq 0 ]; then
        break
    fi

    # start relay
    if [ -f $BRIDGE_TESTS_FOLDER/$TEST_PREFIX-start-relay.sh ]; then
        start_coproc "${BRIDGE_TESTS_FOLDER}/${TEST_PREFIX}-start-relay.sh" "relay"
        RELAY_COPROC=$COPROC_PID
        ((TEST_COPROCS_COUNT++))
    fi
    # start tests
    for zndsl_file in "${zndsl_files[@]}"; do
        start_coproc "$ZOMBIENET_BINARY_PATH --provider native test $zndsl_file" "$zndsl_file"
        echo -n "1">>$TEST_FOLDER/exit-sync
        ((TEST_COPROCS_COUNT++))
    done
    # wait until all tests are completed
    relay_exited=0
    for n in `seq 1 $TEST_COPROCS_COUNT`; do
        wait -n -p COPROC_PID
        exit_code=$?
        coproc_name=${TEST_COPROCS[$COPROC_PID, 0]}
        coproc_log=${TEST_COPROCS[$COPROC_PID, 1]}
        coproc_stdout=$(cat $coproc_log)
        relay_exited=$(expr "${coproc_name}" == "relay")
        echo "Process $coproc_name has finished with exit code: $exit_code"

        # if exit code is not zero, exit
        if [ $exit_code -ne 0 ]; then
            echo "====================================================================="
            echo "=== Shutting down. Log of failed process below                    ==="
            echo "====================================================================="
            echo $coproc_stdout
            exit 1
        fi

        # if last test has exited, exit relay too
        if [ $n -eq $(($TEST_COPROCS_COUNT - 1)) ] && [ $relay_exited -eq 0 ]; then
            kill $RELAY_COPROC
            break
        fi
    done
    ((TEST_INDEX++))
done

echo "====================================================================="
echo "=== All tests have completed successfully                         ==="
echo "====================================================================="
