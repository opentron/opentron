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
  - [x] witness
  - [x] multisig (via raw transaction sign)
  - [ ] system contract handling
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

- [x] AccountUpdateContract - set account name
- [x] AccountPermissionUpdateContract
- [ ] ~~SetAccountIdContract~~
- [ ] ~~AccountCreateContract~~
- [x] FreezeBalanceContract
- [x] UnfreezeBalanceContract
- [x] TransferContract
- [ ] TransferAssetContract
- [ ] VoteAssetContract
- [ ] AssetIssueContract
- [ ] ParticipateAssetIssueContract
- [ ] UnfreezeAssetContract
- [ ] UpdateAssetContract
- [x] VoteWitnessContract
- [x] WitnessCreateContract
- [x] WitnessUpdateContract
- [x] WithdrawBalanceContract - withdraw SR rewards
- [x] CreateSmartContract
- [x] TriggerSmartContract
- [ ] UpdateSettingContract
- [ ] UpdateEnergyLimitContract
- [ ] ClearABIContract
- [x] ProposalCreateContract
- [x] ProposalApproveContract
- [x] ProposalDeleteContract
- [ ] ExchangeCreateContract
- [ ] ExchangeInjectContract
- [ ] ExchangeWithdrawContract
- [ ] ExchangeTransactionContract
- [ ] ShieldedTransferContract
- [ ] ~~BuyStorageContract~~
- [ ] ~~BuyStorageBytesContract~~
- [ ] ~~SellStorageContract~~
- [ ] ~~UpdateBrokerageContract~~

## wallet-cli

A command-line tool which let developers interact Tron Protocol as well as deploy, test smart contracts.
