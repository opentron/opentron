# rust-tron

Rust implementation of ~~the Tron whitepaper~~(wallet-cli only).

## wallet-cli

A command-line tool which let developers interact Tron Protocol as well as deploy, test smart contracts.

- wallet-cli
  - [x] fetch chain status, node, transaction, block, account, contract, asset etc.
  - [x] transfer TRX
  - [x] local wallet management
  - [ ] contract setup
  - [ ] contract calling (including TRC20)
  - [x] accout permission handling
  - [ ] accout resource handling
  - [ ] vote and witness
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
