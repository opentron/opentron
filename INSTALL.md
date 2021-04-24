# Install

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [MacOS](#macos)
- [CentOS 7](#centos-7)
- [Windows](#windows)
- [Other Platforms](#other-platforms)

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

cargo build -p opentron --features static-rocksdb
```

## Windows

You need to have VS 2019 Developer Tools, MSYS2 and vcpkg installed.

```bash
vcpkg install rocksdb[snappy]:x64-windows-static-md

./scripts/download-ztron-params.sh

cargo build
```

## Other Platforms

Java-tron requires `x64(x86_64)` architecture. It is because it uses `java.math` wrongly
and relies on `8087` extended precision float point arithmetic.

For non-x64 architectures, you can still build and run OpenTron, but it won't be compatible with
original java-tron.

This project can be run on following platforms:

- Apple Silicon
- Raspberry Pi 4(or 3+)
