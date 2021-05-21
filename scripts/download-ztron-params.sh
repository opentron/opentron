#!/bin/bash

set -e

mkdir -p ztron-params
cd ztron-params
wget -c https://github.com/tronprotocol/java-tron/raw/master/framework/src/main/resources/params/sapling-output.params
wget -c https://github.com/tronprotocol/java-tron/raw/master/framework/src/main/resources/params/sapling-spend.params
