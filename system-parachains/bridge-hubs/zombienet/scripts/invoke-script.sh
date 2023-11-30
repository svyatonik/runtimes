#!/bin/bash

pushd $FELLOWSHIP_FOLDER/system-parachains/bridge-hubs/zombienet/scripts
./bridges_kusama_polkadot.sh $1
popd
