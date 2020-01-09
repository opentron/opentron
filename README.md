# rust-tron

Rust implementation of ~~the Tron whitepaper~~(wallet-cli only).

## quickstart

```console
> # install rust-nightly
> curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
> rustup component add rustfmt

> # get code
> git clone --recurse-submodules git clone --recurse-submodules https://github.com/andelf/rust-tron.git
> # compile tools
> cd ./rust-tron/
> cargo build -p walletd
> cargo build -p wallet-cli

> # time to rock !!!
> ./target/debug/wallet-cli --help

> # or use testnet toolset
> ./nile-wallet-cli.sh
```

## TODOs

- wallet-cli
  - [ ] shielded transaction
    - [x] demo works
    - [ ] shielded notes management
    - [ ] real subcommand

- [ ] full Tron Protocol implementation
  - [x] joking
  - [ ] network
  - [ ] p2p
  - [ ] rpc
  - [ ] consensus
  - [ ] chain
  - [ ] chainbase
  - [ ] governance
  - [ ] sync
  - [ ] script
    - [ ] solidity
    - [ ] wasm

### All System Contracts Support and corresponding wallet-cli commands

- [x] AccountUpdateContract - set account name
  - set account_name
- [x] AccountPermissionUpdateContract
  - set account_permission
- [x] FreezeBalanceContract
  - system freeze
- [x] UnfreezeBalanceContract
  - system unfreeze
- [x] TransferContract
  - transfer
- [x] TransferAssetContract
  - asset transfer
- [x] AssetIssueContract
  - asset issue
- [x] ParticipateAssetIssueContract
  - asset participate
- [ ] UnfreezeAssetContract
- [x] UpdateAssetContract
  - asset update
- [x] VoteWitnessContract
  - system vote_witness
- [x] WitnessCreateContract
  - system create_witness
- [x] WitnessUpdateContract
  - system update_witness
- [x] WithdrawBalanceContract - withdraw SR rewards
  - system withdraw_reward
- [x] CreateSmartContract
  - contract create
- [x] TriggerSmartContract
  - contract call
- [x] UpdateSettingContract
  - contract update
- [x] UpdateEnergyLimitContract
  - contract update
- [x] ClearABIContract
  - contract clear_abi
- [x] ProposalCreateContract
  - system create_proposal
- [x] ProposalApproveContract
  - system approve_proposal
  - system disapprove_proposal
- [x] ProposalDeleteContract
  - system delete_proposal
- [ ] ExchangeCreateContract
- [ ] ExchangeInjectContract
- [ ] ExchangeWithdrawContract
- [ ] ExchangeTransactionContract
- [ ] ShieldedTransferContract
- [ ] ~~BuyStorageContract~~
- [ ] ~~BuyStorageBytesContract~~
- [ ] ~~SellStorageContract~~
- [ ] ~~UpdateBrokerageContract~~
- [ ] ~~SetAccountIdContract~~
- [ ] ~~AccountCreateContract~~
- [ ] ~~VoteAssetContract~~

## wallet-cli

A command-line tool which let developers interact Tron Protocol as well as deploy, test smart contracts.

> NOTE: Always use ``--help`` to get hint about how to use the command.

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
    -r, --ref-block <ref-block>            Set the reference block num or block id used for TAPOS (Transaction as Proof-
                                           of-Stake)
```
