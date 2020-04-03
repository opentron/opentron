# EVM

## Functions

Library functions cannot be payable.

Functions cannot be constant and payable at the same time.

Internal functions cannot be payable.

_stateMutability 替代了 _isDeclaredConst, _isPayable

## Events

ref: <https://sawtooth.hyperledger.org/docs/core/releases/0.8/solidity_developers_guide/seth_transaction_receipts_and_events.html>
ref: <https://solidity.readthedocs.io/en/v0.4.24/contracts.html#low-level-interface-to-logs>

Events allow the convenient usage of the EVM logging facilities.

Events are inheritable members of contracts. When they are called, they cause the arguments to be stored in the transaction’s log - a special data structure in the blockchain.

EVM event = log0, log1, log2, log3 and log4.
log0(_id); // the event id with out any parameter

The first argument will be used for the data part of the log and the others as topics.

```protobuf
// Protobuf message to store in the Sawtooth Event's event_data field
message EvmLogData {
    // 20 Bytes - address from which this log originated.
    bytes address = 1;

    // Array of 0 to 4 32-byte blobs of data.
    // (In solidity: The first topic is the hash of the signature
    // of the event (e.g. Deposit(address,bytes32,uint256)), except
    // you declared the event with the 'anonymous' specifier.)
    // See the following:
    // https://github.com/ethereum/wiki/wiki/JSON-RPC#eth_getfilterchanges
    repeated bytes topics = 2;

    // contains one or more 32 Bytes non-indexed arguments of the log.
    bytes data = 3;
}
```

ref: <https://github.com/paritytech/substrate/blob/master/frame/evm/src/backend.rs>

```rust
pub struct Log {
    /// Source address of the log.
    pub address: H160,
    /// Topics of the log.
    pub topics: Vec<H256>,
    /// Byte array data of the log.
    pub data: Vec<u8>,
}
```

contract: TXnBP1MHkYww9R9mrAgcZ8i4t5TKvFnXWy
txn id: 605d7130028c116d280682faf6f2a73ae0aaa684bb81d978e102eae0f58affef

```json
[
  {
    "address": "ef3cd68915b0aaf61d4301eeb0a9222c9730bee4",
    "data": "0000000000000000000000009c7e0a1ab7a5eb3b9ba6c7d5e69abb7ba392704f0000000000000000000000000000000000000000000000000000000004c4b400000000000000000000000000000000000000000000000000000000000000005d0000000000000000000000000000000000000000000000000000000000000001000000000115151674e0819c1f30252f20a761efc29e52db1483f47e26c3bb39000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000004066303139363737313861326265666261396266666230656135663363313536666364636435323936313165643234343238366133626537326431663232383737",
    "topics": [
      "ea4f248a9a34b0ef200e6f5463b313b5568faaf8b073b8b541e16f9a2110c817"
    ]
  }
]
```

event diceResult(address user, uint256 betValue, uint256 prediction, uint256 is_under, bytes32 blockHash, string tx, bool win, uint256 lucky_number)
    => ea4f248a9a34b0ef200e6f5463b313b5568faaf8b073b8b541e16f9a2110c817

```js
event Deposit(
  address indexed _from,
  bytes32 indexed _id,
  uint _value
);
```

Indexed arguments will not be stored themselves. You can only search for the values,
but it is impossible to retrieve the values themselves. 在data字段会被忽略，通过 topics 字段保存.

```text
txn id: b4264ed1e516724bdbdde36cba1e0adde71b843abffe20c98efe718e23e61756
! Sender Address(base58check):   TCu2ovwasJNKvd4tefqo3toKaX8cvC4Qyc
! Contract Address(base58check): TWmhXhXjgXTj87trBufmWFuXtQP8sCWZZV
! function GoodLuck(uint256 _number, uint256 _direction) payable
!          GoodLuck(uint256,uint256) [a3082be9]
! Arguments:
  _number: uint256 = 50
  _direction: uint256 = 0

event DiceCreate(uint256 indexed _orderId, address indexed _bettor, uint256 _number, uint256 _direction, uint256 _amount)
    => 4a0276f32d1dc1f56ff4e6491876a2f2512eb9ed7e9da8307716b6204e3c6340

[
  {
    "address": "e42d76d15b7ecd27a92cc9551738c2635c63b71c",
    "data": "00000000000000000000000000000000000000000000000000000000000000320000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001312d000",
    "topics": [
      "4a0276f32d1dc1f56ff4e6491876a2f2512eb9ed7e9da8307716b6204e3c6340",
      "00000000000000000000000000000000000000000000000000000000000089bf",
      "000000000000000000000000201e34acefe5b39fcc01d261e7c3e7b9c1a54502"
    ]
  }
]
```
