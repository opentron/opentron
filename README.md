# rust-tron

Rust implementation of ~~the Tron whitepaper~~(wallet-cli only).

## quickstart

```console
# install rust-nightly
> curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
> rustup component add rustfmt
# compile
> cargo build -p walletd
> cargo build -p wallet-cli
# time to rock !!!
> ./target/debug/wallet-cli help
# or use testnet toolset
> ./nile-wallet-cli.sh
```

## TODOs

- wallet-cli
  - [x] fetch chain status, node, transaction, block, account, contract, asset etc.
  - [x] transfer TRX
  - [x] local wallet management
  - [x] contract setup
  - [x] contract calling (including TRC20)
  - [x] accout permission handling
  - [x] accout resource handling
  - [x] vote
  - [ ] witness
  - [x] multisig (via raw transaction sign)
  - [ ] raw transaction handling
    - [ ] create
    - [x] sign
  - [ ] shielded transaction
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

### All Contracts Support

- [ ] AccountCreateContract
- [x] AccountUpdateContract - set account name
- [ ] ~~SetAccountIdContract~~
- [x] TransferContract
- [ ] TransferAssetContract
- [ ] VoteAssetContract
- [x] VoteWitnessContract
- [ ] UpdateSettingContract
- [ ] UpdateEnergyLimitContract
- [ ] ClearABIContract
- [ ] WitnessCreateContract
- [ ] WitnessUpdateContract
- [ ] AssetIssueContract
- [ ] ParticipateAssetIssueContract
- [x] FreezeBalanceContract
- [x] UnfreezeBalanceContract
- [ ] UnfreezeAssetContract
- [ ] WithdrawBalanceContract
- [ ] UpdateAssetContract
- [ ] ProposalCreateContract
- [ ] ProposalApproveContract
- [ ] ProposalDeleteContract
- [x] CreateSmartContract
- [x] TriggerSmartContract
- [ ] ~~BuyStorageContract~~
- [ ] ~~BuyStorageBytesContract~~
- [ ] ~~SellStorageContract~~
- [ ] ExchangeCreateContract
- [ ] ExchangeInjectContract
- [ ] ExchangeWithdrawContract
- [ ] ExchangeTransactionContract
- [x] AccountPermissionUpdateContract
- [ ] ShieldedTransferContract
- [ ] ~~UpdateBrokerageContract~~


## wallet-cli

A command-line tool which let developers interact Tron Protocol as well as deploy, test smart contracts.
