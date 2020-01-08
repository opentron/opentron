# Contract reference

## Creating Smart Contract on Chain

```console
> # compile contract
> solc -o . --bin Contract.sol
> solc -o . --abi Contract.sol

> # create contract
> ./target/debug/wallet-cli contract create TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
>     --abi ./Contract.abi --code ./Contract.bin \
>     --name YourSuperCoolContractName
.... (omitted)
TX: d8228648cb275bb548da9a8b2d11beb956275413774015a63e390de85bc1fb57
.... (omitted)

> # get contract address
> ./target/debug/wallet-cli get transaction_info \
>     d8228648cb275bb548da9a8b2d11beb956275413774015a63e390de85bc1fb57
.... (omitted)
"contract_address": "4123ff5e2eac2926b5ed72948eb9e69a07f24b49ba",
.... (omitted)

> # get contract ABI
> ./target/debug/wallet-cli get contract 4123ff5e2eac2926b5ed72948eb9e69a07f24b49ba
.... (omitted)
function get() returns (uint256)
    => 6d4ce63c: get()
function set(uint256 x)
    => 60fe47b1: set(uint256)

> # call contract
> ./target/debug/wallet-cli contract call TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
>     4123ff5e2eac2926b5ed72948eb9e69a07f24b49ba \
>     'set(uint256)' -- 20

> # use `get transaction_info` to fetch calling result

> ./target/debug/wallet-cli contract call TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
>     4123ff5e2eac2926b5ed72948eb9e69a07f24b49ba \
>     'get(uint256)'
.... (omitted)
TX: 26ff621bcb2c74da8a14c77c20d55cbad8cb0ea022e1174d9522c283c133355c
.... (omitted)
> ./target/debug/wallet-cli get transaction_info \
>     26ff621bcb2c74da8a14c77c20d55cbad8cb0ea022e1174d9522c283c133355c
.... (omitted)
  "contractResult": [
    "0000000000000000000000000000000000000000000000000000000000000014"
  ],
.... (omitted)
```

## ContractType Code

```c
AccountCreateContract = 0;
TransferContract = 1;
TransferAssetContract = 2;
VoteAssetContract = 3;
VoteWitnessContract = 4;
WitnessCreateContract = 5;
AssetIssueContract = 6;
WitnessUpdateContract = 8;
ParticipateAssetIssueContract = 9;
AccountUpdateContract = 10;
FreezeBalanceContract = 11;
UnfreezeBalanceContract = 12;
WithdrawBalanceContract = 13;
UnfreezeAssetContract = 14;
UpdateAssetContract = 15;
ProposalCreateContract = 16;
ProposalApproveContract = 17;
ProposalDeleteContract = 18;
SetAccountIdContract = 19;
CustomContract = 20;
CreateSmartContract = 30;
TriggerSmartContract = 31;
GetContract = 32;
UpdateSettingContract = 33;
ExchangeCreateContract = 41;
ExchangeInjectContract = 42;
ExchangeWithdrawContract = 43;
ExchangeTransactionContract = 44;
UpdateEnergyLimitContract = 45;
AccountPermissionUpdateContract = 46;
ClearABIContract = 48;
UpdateBrokerageContract = 49;
ShieldedTransferContract = 51;
```
