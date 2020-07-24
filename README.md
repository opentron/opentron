# OpenTron

OpenTron is an implementation of the Tron blockchain written in Rust. This project is under active development and is
not ready for general use.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Rationale](#rationale)
- [Roadmap](#roadmap)
  - [TODOs](#todos)
- [Quickstart](#quickstart)
- [wallet-cli](#wallet-cli)
  - [All System Contracts Support and corresponding wallet-cli commands](#all-system-contracts-support-and-corresponding-wallet-cli-commands)
  - [Chain Lookup](#chain-lookup)
  - [Local Wallet Management](#local-wallet-management)
  - [Common Transaction Options](#common-transaction-options)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Rationale

1. Decentralised

   The Tron network currently has [one and only implementation](https://github.com/tronprotocol/java-tron). This
   has lead to some criticism of Tron being too centralized. We want to change that by providing an alternative
   implementation and independent development team.

2. High performance

   API calls to java-tron nodes often results in CPU timeouts and other "out of resource" related errors. This is partly
   due to java-tron being written in Java, a garbage collected language that runs on a virtual machine. OpenRust is
   written in Rust, a modern compiled language that is increasingly adopted for blockchain and systems development due
   to its high performance, safety and modern design.

3. Modern codebase

   Java-tron was forked a few years ago from a Java Ethereum implementation. It has accumulated a lot of technical debt
   over the years and has a lot of inconsistent or missing documentation. We believe that a greenfield implementation
   will enable us to produce a cleaner code base that is easier to understand, improve and extend. In addition, since
   Rust has first class support for WebAssembly, it will be possible to re-use its code for creating web based clients,
   wallets, explorers, etc.

## Roadmap

- [x] wallet-cli (the full feature wallet/rpc client)
  - [x] walletd (the wallet daemon)
- [ ] node-cli, under active development

  - [x] sync with java-tron node
  - [ ] transaction handling

- [x] Block data sync, only blocks (raw transactions), without transaction info and any other state data. Handle chain fork and block Merkle tree verification.
- [ ] Simple transaction broadcast, without much verification, just broadcast transactions to the network as quickly as possible(an airdrop tool can be made from it)
- [ ] Handle transaction verification. all state data will be available. (difficult, EVM engine, resource consumption mode, witness/vote/proposal, chain parameter are all handled at this stage, to make the state data identical as java-tron )
- [ ] Build a query API layer upon state data. json-rpc.
- [ ] Build a event API layer upon state data.
- [ ] block mining logic (difficult, DPoS mining, need resource to become an SR)

### TODOs

- [x] wallet-cli

  - [ ] shielded transaction
    - [x] demo works
    - [ ] shielded notes management
    - [ ] real subcommand

- [ ] Full Tron Protocol implementation
  - [x] proto2: refactor the protobuf / ~~gRPC~~
  - [x] primitives
    - [x] use primitive-types
    - [x] Address, PublicKey, PrivateKey, Signature
  - [ ] config file
    - [x] genesis block parsing
    - [x] toml config file parsing
    - [ ] organize chain parameters
  - [ ] discover protocol
    - [x] demo works
  - [ ] channel protocol
    - [x] demo works
    - [x] sync
    - [ ] minor bug fix, timeout error
  - [ ] chain
    - [x] Block / Transaction
  - [ ] chainbase
    - [ ] memory
    - [ ] RocksDB
  - [ ] mempool
  - [ ] consensus
  - [ ] EVM / TVM
    - [x] 3.7 TVM <https://github.com/andelf/evm>
    - [x] 4.0 TVM with zksnark: `ztron` crate
    - [ ] massive tests
  - [ ] RPC replacement
    - will not support gRPC
    - might have json-rpc support
  - [ ] shielded transaction
    - [ ] ztron
  - [ ] SM2 / SM3 support
    - [x] [sm2](https://docs.rs/sm2/)
    - [x] [sm3](https://docs.rs/sm3/)

## Quickstart

See [INSTALL.md](./INSTALL.md) for more detailed information.

```console
> # Install rust-nightly
> curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
> rustup component add rustfmt

> # Install protoc
> brew install protobuf  # macOS
> sudo pacman -S protobuf  # ArchLinux
> sudo apt install protobuf-compiler libprotobuf-dev # Ubuntu / Debian

> # Install RocksDB
> brew install rocksdb  # macOS

> # Get code
> git clone --recurse-submodules https://github.com/oikos-cash/OpenTron.git

> # Compile tools
> cd ./OpenTron/
> cargo build --all

> # Time to rock!!!
> ./target/debug/wallet-cli --help

> # or use testnet toolset
> ./nile-wallet-cli.sh
```

## wallet-cli

A command-line tool which let developers interact Tron Protocol as well as deploy, test smart contracts.

> NOTE: Always use `--help` to get hint about how to use the command.

### All System Contracts Support and corresponding wallet-cli commands

- Account
  - AccountUpdateContract: `set account_name`
  - AccountPermissionUpdateContract: `set account_permission`
  - FreezeBalanceContract: `system freeze`
  - UnfreezeBalanceContract: `system unfreeze`
- Transfer: `transfer`
- TRC10 Asset
  - TransferAssetContract: `asset transfer`
  - AssetIssueContract: `asset issue`
  - UpdateAssetContract: `asset update`
  - ParticipateAssetIssueContract: `asset participate`
  - UnfreezeAssetContract: `asset unfreeze`
- SmartContract
  - CreateSmartContract: `contract create`
  - TriggerSmartContract: `contract call`
  - UpdateSettingContract: `contract update`
  - UpdateEnergyLimitContract: `contract update`
  - ClearABIContract: `contract clear_abi`
- Witness
  - VoteWitnessContract: `system vote_witness`
  - WitnessCreateContract: `system create_witness`
  - WitnessUpdateContract: `system update_witness`
  - WithdrawBalanceContract: `system withdraw_reward` - withdraw SR rewards
  - UpdateBrokerageContract: `system update_brokerage`
- Proposal
  - ProposalCreateContract: `system create_proposal`
  - ProposalApproveContract: `system approve_proposal` `system disapprove_proposal`
  - ProposalDeleteContract: `system delete_proposal`
- Exchange
  - [ ] ExchangeCreateContract
  - [ ] ExchangeInjectContract
  - [ ] ExchangeWithdrawContract
  - [ ] ExchangeTransactionContract
- ShieldedTransfer
  - [ ] ShieldedTransferContract
- ~~Deprecated~~
  - Storage deprecated
    - BuyStorageContract
    - BuyStorageBytesContract
    - SellStorageContract
  - Deprecated account
    - SetAccountIdContract - id useless?
    - AccountCreateContract - transfering creates account
  - VoteAssetContract

### Chain Lookup

```text
get subcommand
    account               Retrieve an account from the blockchain
    account_permission    Retrieve account permision info from the blockchain
    account_resource      Retrieve energy and bandwidth usage of an account
    asset                 Get details of a TRC10 token
    block                 Retrieve a full block from the blockchain
    brokerage             Get brokerage info, voting sharing ratio
    contract              Get details of a smart contract
    node                  Get current connected node state information
    proposal              Get details of a proposal
    reward                Get reward info, the unwithdrawn voting reward
    transaction           Retrieve a transaction from the blockchain
    transaction_info      Retrieve receipt of atransaction

list subcommand
    asset        Retrieve list of all tokens
    exchange     Retrive list of all exchanges
    node         List the nodes which is connecting to the network
    parameter    List chain parameters
    proposal     Retrive list of all proposals
    witness      Query the list of Super Representatives
```

### Local Wallet Management

```console
> ./target/debug/wallet-cli wallet create --password YOUR_PASSWORD
Wallet created!
> ./target/debug/wallet-cli wallet open
Wallet opened!
> ./target/debug/wallet-cli wallet create_key  # create a random key
Address: TMd3p......4JstUBsxen
Public:  f19f2d5b2c4067a.......7f0d1faaa094a23704d9
Private: 884f5218eb5da..............6809aa8da19d406
> ./target/debug/wallet-cli wallet keys  # list all keys
......(omitted)......
> ./target/debug/wallet-cli wallet --help  # get help
```

### Common Transaction Options

```text
FLAGS:
    -d, --dont-broadcast    Don't broadcast transaction to the network (just print to stdout)
    -h, --help              Prints help information
    -s, --skip-sign         Skip actual sign process

OPTIONS:
    -k, --account <account>                The account address used for signing
    -x, --expiration <expiration>          Set the time in seconds before a transaction expires
        --fee-limit <fee-limit>            Maximum value of TRX allowed consumed
        --permission-id <permission-id>    Permission id used by transaction [possible values: 0, 2, 1]
    -K, --private-key <private-key>        The private key used for signing
    -r, --ref-block <ref-block>            Set the reference block num or block id used for TAPOS
```
