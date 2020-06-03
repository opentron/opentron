# The node-cli

## UDP discovery protocol

discover port: 18888/udp

```rust
// UDP message type
const DISCOVER_PING: u8 = 0x01;
const DISCOVER_PONG: u8 = 0x02;
const DISCOVER_FIND_NEIGHBORS: u8 = 0x03;
const DISCOVER_NEIGHBORS: u8 = 0x04;
const BACKUP_KEEP_ALIVE: u8 = 0x05;
```

```text
=> PING
<= PONG
<= PING  # only if version check passed, and new peer found
=> PONG
=> FIND
<= NEIGHBOURS
<= FIND
=> NEIGHBOURS
```

## TCP channel protocol

### Message types

```rust
// HandshakeHello = 0x20,
// HandshakeDisconnect = 0x21,

// Ping = 0x22,
// Pong = 0x23,

// SyncBlockchain = 0x08,
// BlockchainInventory = 0x09,

// Inventory = 0x06,
// FetchInventoryData = 0x07,

// Block = 0x02,
// Transactions = 0x03,

// Transaction = 0x01,
// Blocks = 0x04,
// BlockHeaders = 0x05,
// ItemNotFound = 0x10,
// FetchBlockHeaders = 0x11,
// BlockInventory = 0x12,
// TransactionInventory = 0x13,
// DiscoverPing = 0x30,
// DiscoverPong = 0x31,
// DiscoverFindPeer = 0x32,
// DiscoverPeers = 0x33,
```

### Handshake

File: HandshakeHandler.java

#### normal handshake

=> P2P_HELLO(0x20),      (version, genesis block, head block)
<= P2P_HELLO(0x20),
(handshake finished)

#### disconnect handshake

=> P2P_HELLO(0x20),       HelloMessage
<= P2P_DISCONNECT(0x21),  DisconnectMessage

Reason might be:

- UNEXPECTED_IDENTITY
  - while set up as a FastForward, only accept connection from witness
- TOO_MANY_PEERS
  - backward compatiblity, only when remoteId.length != 64
- INCOMPATIBLE_VERSION
  - p2p version mismatch
- INCOMPATIBLE_CHAIN
  - genesis block mismatch
- FORKED
  - peer's solid block <= local solid block
  - peer's solid block not in local store

Then handshake OK.

### ping & pong

=> P2P_PING(0x22),  PingMessage, FIXED_PAYLOAD = Hex.decode("C0");
<= P2P_PONG(0x23),  PongMessage, FIXED_PAYLOAD = Hex.decode("C0");
<= P2P_PING
=> P2P_PONG

PING without response will cause connection close.

Too many PING/PONG in 10s, PING/PONG mishmatch will cause a BAD_PROTOCOL (P2pHandler.java).

### syncing from remote nodes

- => SYNC_BLOCK_CHAIN(0x08),        BlockInventory
  - getBlockChainSummary (block ids from forks)
  - log SyncChainRequested (ids, timestamp)
  - send SyncBlockChainMessage
- <= BLOCK_CHAIN_INVENTORY(0x09),   ChainInventory  (ChainInventoryMsgHandler.java)
  - get a list of block ids
  - check BAD_MESSAGE(BAD_PROTOCOL)  `check()`
    - haven't send a sync request
    - len(ids) must > 0
    - len(ids) must <= 2000 + 1  `SYNC_FETCH_BATCH_NUM`
    - has remain_num, then len(ids) must = 2001
    - ids are increasing 1 by 1
    - ids[0] must be one of requested in sync reqest
    - max future num check, 3s/block, > current time
  - if len(ids) == 1 and in db, stop sync from peer
- => FETCH_INV_DATA(0x07),          Inventory [BlockID], default batch=100
- <= BLOCK(0x02),
- <= BLOCK(0x02),
- <= BLOCK(0x02),
- ....
- => FETCH_INV_DATA(0x07),          Inventory [BlockID]

依次循环，直到开始 INVENTORY 逻辑

### TIME_OUT 逻辑

特殊情况? 发现似乎是有单独检测 TimeOut 的线程? 独立于 ping/pong. 30s 超时.
`PeerStatusCheck.java`

- getBlockBothHaveUpdateTime < now - (blockUpdateTimeout = 30s)
  - 通过 setBlockBothHave 更新，直接记录当前时间
  - 更新场景:
    - BlockMsgHandler: 必须 parentId 存在, 否则 unlink 错误
      - 本地必须是发送过 sync block request, 或者 advertise block inventory
        - 但似乎好像是。。。没发过也可以，会从 null 置为 0
    - ChainInvMsgHandler: ...
    - FetchInvDataMsgHandler: 对方 fetch 最新 block_inv 的时候, 更新
- getAdvInvRequest < now - (ADV_TIME_OUT = 20s)
- getSyncBlockRequested < now - (SYNC_TIME_OUT = 5s)
  - 发送 FetchBlockInventory 后 5s 必须有返回

但还是无解。未找到核心 TimeOut 逻辑。

getLostBlockIds: unForkId

```text
BLOCK(0x02),

TRX(0x01),
BLOCKS(0x04),
TRXS(0x03),
BLOCKHEADERS(0x05),

# sync a new block
<= INVENTORY(0x06, type=BLOCK),
=> FETCH_INV_DATA(0x07, type=BLOCK),
<= BLOCK(0x02)
=> others INVENTORY(0x06)

ITEM_NOT_FOUND(0x10),
FETCH_BLOCK_HEADERS(0x11),
BLOCK_INVENTORY(0x12),
TRX_INVENTORY(0x13),

unused:

DISCOVER_PING(0x30),
DISCOVER_PONG(0x31),
DISCOVER_FIND_PEER(0x32),
DISCOVER_PEERS(0x33),
```

## Channel Messages

BLOCK: BlockMessage (syncService)
    Block
TRXS: TransactionsMessage (trxHandlePool)
    Transactions
INVENTORY: InventoryMessage (AdvService)
    Inventory
FETCH_INV_DATA: FetchInvDataMessage
    Inventory(type=BLOCK) <= [Block]
    Inventory(type=TRX) <= [Transactions] MAX_SIZE=1_000_000
SYNC_BLOCK_CHAIN: SyncBlockChainMessage
    BlockInventory(type=SYNC) <= ChainInventory(ids, remain_num: 17420239)

    //len(ids) ==
BLOCK_CHAIN_INVENTORY: ChainInventoryMessage (syncService)
    ChainInventory
TRX: TransactionMessage, not used
    Transaction
BLOCKS: BlocksMessage, not used
    Items
ITEM_NOT_FOUND: not used
    Items(type=ERR)
FETCH_BLOCK_HEADERS: FetchBlockHeadersMessage, not used
    Inventory
TRX_INVENTORY: TransactionInventoryMessage, not used
    Inventory

## Net Message that is using
case SYNC_BLOCK_CHAIN:
  syncBlockChainMsgHandler.processMessage(peer, msg);
case BLOCK_CHAIN_INVENTORY:
  chainInventoryMsgHandler.processMessage(peer, msg);
case INVENTORY:
  inventoryMsgHandler.processMessage(peer, msg);
case FETCH_INV_DATA:
  fetchInvDataMsgHandler.processMessage(peer, msg);
case BLOCK:
  blockMsgHandler.processMessage(peer, msg);
case TRXS:
  transactionsMsgHandler.processMessage(peer, msg);


## SyncService

state = SYNCING
remain_num = 0
block_id_both_have = genesis_block_id
