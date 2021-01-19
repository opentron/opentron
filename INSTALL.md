# Install

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Install](#install)
  - [MacOS](#macos)
  - [CentOS 7](#centos-7)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## MacOS

```bash
# Install RocksDB (https://github.com/bh1xuw/rust-rocks#how-to-compile)
brew install rocksdb

# Install Protobuf
brew install protobuf cmake wget

# Clone opentron
git clone https://github.com/opentron/opentron.git
cd opentron

# Download ztron params
./scripts/download-ztron-params.sh

# build all
cargo build
# run tests
cargo test
```

## CentOS 7

```bash
# (This official repo's protobuf-compiler is out-dated.)
# (Unrecognized syntax identifier "proto3".  This parser only recognizes "proto".)

wget https://github.com/protocolbuffers/protobuf/releases/download/v3.11.4/protoc-3.11.4-linux-x86_64.zip
unzip protoc-3.11.4-linux-x86_64.zip
sudo cp -rv bin include /usr/local

git clone --recurse-submodules https://github.com/opentron/opentron.git
cd opentron

# Download ztron params
./scripts/download-ztron-params.sh

cargo build
```
