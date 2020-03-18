# All Stores

## 当前 stores

### account

[u8; 21]
address => Account

### account-index

account name => address

### accountid-index

account id in lower case => address

### asset-issue / asset-issue-v2

取决于 getAllowSameTokenName

key => AssetIssueContract

asset-issue:
key = AssetIssueContract.name

asset-issue-v2:
key = AssetIssueContract.id

### witness

address => Witness

```json
{
    "address": "4138e3e3a163163db1f6cfceca1d1c64594dd1f0ca",
    "isJobs": true,
    "latestBlockNum": 18068601,
    "latestSlotNum": 528155924,
    "pubKey": [],
    "totalMissed": 1714,
    "totalProduced": 617923,
    "url": "https://twitter.com/justinsuntron",
    "voteCount": 309266568
  }
```

由 latestBlockNum 来计算 solid block number.
所有 witness 的 latestBlockNum ，取第

```java
(int) (size * (1 - SOLIDIFIED_THRESHOLD * 1.0 / 100)); // 个
public static final int SOLIDIFIED_THRESHOLD = 70; // 70%
```

该值在 applyBlock() 函数中更新：

```java
wc.setTotalProduced(wc.getTotalProduced() + 1);
wc.setLatestBlockNum(blockNum);
wc.setLatestSlotNum(dposSlot.getAbSlot(blockTime));
```

dpos_slot 在 block_num = 1 时候 =1

然后 applyBlock() 函数处理 miss block 情况

AbSlot = (当前时间 - genesis_block 时间) / BLOCK_PRODUCED_INTERVAL

latest_slot_num  x 3 基本就是上次出块时间

SR数确认 ./mainnet-wallet-cli.sh list witness | grep isJobs | grep true | wc -l

### witness_schedule

21 字节连续编码存地址

```text
"active_witnesses" =>  List<ByteString>
"current_shuffled_witnesses" => List<ByteString>
```

### properties

dynamic properties

```text
latest_block_header_timestamp
latest_block_header_number
latest_block_header_hash

LATEST_SOLIDIFIED_BLOCK_NUM
LATEST_PROPOSAL_NUM
LATEST_EXCHANGE_NUM
.....
ACCOUNT_UPGRADE_COST
WITNESS_PAY_PER_BLOCK
TRANSACTION_FEE
ASSET_ISSUE_FEE
MULTI_SIGN_FEE

TOKEN_ID_NUM
```

### proposal

```text
[u8; 8] => Proposal
ByteArray.fromLong(proposal_id)  => Proposal
```

```json
{
    "approvals": [
      "41ff324071970b2b08822caa310c1bb458e63a5033",
      "410c4c64201f66a32719cf9ab4e6f4aed6330b48bd"
    ],
    "create_time": 1582027374000,
    "expiration_time": 1582028400000,
    "parameters": {
      "29": 100
    },
    "proposal_id": 17,
    "proposer_address": "41ff324071970b2b08822caa310c1bb458e63a5033",
    "state": "DISAPPROVED"
  }
```

### votes

```text
[u8; 21] => Votes
voter address => Votes
```

```protobuf
message Votes {
  bytes address = 1;
  repeated Vote old_votes = 2;
  repeated Vote new_votes = 3;
}
```

maintenance 周期计票用.
计票后删除 key

日志特征 `grep 'new votes in this epoch'`

### code

address => byte[]

### contract

address => SmartContract

### block

```text
[u8; 32] => Block
block id => Block
```

### block-index

```text
[u8; 8] => [u8; 32]
block num => block id
```

### DelegatedResource

```text
[u8; 21+21] => DelegatedResource
from address + to address => DelegatedResource
```

### DelegatedResourceAccountIndex

DelegatedResourceAccountIndex.account => DelegatedResourceAccountIndex

```protobuf
message DelegatedResourceAccountIndex {
  bytes account = 1;
  repeated bytes fromAccounts = 2;
  repeated bytes toAccounts = 3;
}
```

### delegation

```
{cycle}-{hex(address)}-vote: long as [u8]
{cycle}-{hex(address)}-reward: long as [u8]
{cycle}-{hex(address)}-account-vote: Account
end-{hex(address)}: long as [u8]
{cycle}-{hex(address)}-brokerage: int as [u8]
```
傻逼设计

### exchange / exchange-v2

exchange id long as [u8] => Exchange

getAllowSameTokenName

exchange 只支持 allow 之前的

### peers ? common

目测再没有使用

### recent-block

[u8; 2] => [u8; 8]

对应
"ref_block_bytes": "__",
"ref_block_hash": "________",

### storage-row

StorageRowCapsule

For EVM, Storage is a persistent associative map, with uint256s as keys and uint256s as values.

index = '0000000000000000000000000000000000000000000000000000000000000005'
key =  '00000000000000000000000xbccc714d56bc0da0fd33d96d2a87b680dd6d0df6'
let newKey =  web3.sha3(key + index, {"encoding":"hex"})

The variables are generally sequential, the first variable declared is in position 0, the second in position 1, etc.

For dynamic arrays, their position contains the length of the array, and the data starts at sha3(position)

ref: https://medium.com/aigang-network/how-to-read-ethereum-contract-storage-44252c8af925

```text
DataWord: key => value
addrHash = sha3(address + trxHash)
key = compose(key, addrHash)
   PREFIX_BYTES = 16;
   addrHash[:16] + key[16:]


storage.generateAddrHash(contract.getTrxHash());
storage = new Storage(address, getStorageRowStore());

调用者分析 VM.java

stack = [VALUE, ADDR, SSTORE]
入口
program.storageSave(addr, value);
getContractState()
 .putStorageValue(MUtil.convertToTronAddress(getContractAddress().getLast20Bytes()), keyWord, valWord);

=> Repository.putStorageValue

// HashMap<Key, Storage> storageCache

storage = storageCache.get(addressKey);
storage.put(key, value);

public Storage(byte[] address, StorageRowStore store) constructor
(set addrhash)
```

### trans

TransactionStore

transaction hash/id => block num as string as bytes fit in [u8; 8] (新)

transaction hash/id => Transaction (旧)

### transactionHistoryStore

transaction hash/id => TransactionInfo

### transactionRetStore

block num as [u8; 8] => TransactionRet

```protobuf
message TransactionRet {
  int64 blockNumber = 1;
  int64 blockTimeStamp = 2;
  repeated TransactionInfo transactioninfo = 3;
}
```

设计真傻逼, 单一区块 transactions 超多时, 是个顺序遍历

## rethink stores

IndexedBlockHeader + IndexedTransaction
