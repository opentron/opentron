# Install

## CentOS 7

```console
(This official repo's protobuf-compiler is out-dated.)
(Unrecognized syntax identifier "proto3".  This parser only recognizes "proto2".)

$ wget https://github.com/protocolbuffers/protobuf/releases/download/v3.11.4/protoc-3.11.4-linux-x86_64.zip
$ unzip protoc-3.11.4-linux-x86_64.zip
$ sudo cp -rv bin include /usr/local

$ git clone https://github.com/oikos-cash/OpenTron.git
$ cd rust-tron

$ cargo build --all



```
