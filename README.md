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

### All System Contracts Support

- [x] AccountUpdateContract - set account name
    - set account_name
- [x] AccountPermissionUpdateContract
    - set account_permission
- [ ] ~~SetAccountIdContract~~
- [ ] ~~AccountCreateContract~~
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
- [ ] ParticipateAssetIssueContract
- [ ] UnfreezeAssetContract
- [ ] UpdateAssetContract
- [ ] ~~VoteAssetContract~~
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

## wallet-cli

A command-line tool which let developers interact Tron Protocol as well as deploy, test smart contracts.
