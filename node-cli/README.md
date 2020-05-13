# The node-cli

## UDP discover

discover: 18888/udp

// UDP message type
const DISCOVER_PING: u8 = 0x01;
const DISCOVER_PONG: u8 = 0x02;
const DISCOVER_FIND_NEIGHBORS: u8 = 0x03;
const DISCOVER_NEIGHBORS: u8 = 0x04;
const BACKUP_KEEP_ALIVE: u8 = 0x05;

```
=> PING
<= PONG
<= PING  # only if version check passed, and new peer found
=> PONG
=> FIND
<= NEIGHBOURS
<= FIND
=> NEIGHBOURS
```

## TCP handshake, sync

```rust
// Transaction = 0x01,
// Block = 0x02,
// Transactions = 0x03,
// Blocks = 0x04,
// BlockHeaders = 0x05,
// Inventory = 0x06,
// FetchInventoryData = 0x07,
// SyncBlockchain = 0x08,
// BlockchainInventory = 0x09,
// ItemNotFound = 0x10,
// FetchBlockHeaders = 0x11,
// BlockInventory = 0x12,
// TransactionInventory = 0x13,
//
// HandshakeHello = 0x20,
// HandshakeDisconnect = 0x21,
//
// Ping = 0x22,
// Pong = 0x23,
//
// DiscoverPing = 0x30,
// DiscoverPong = 0x31,
// DiscoverFindPeer = 0x32,
// DiscoverPeers = 0x33,
```

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

# sync from other nodes
=> SYNC_BLOCK_CHAIN(0x08),    BlockInventory
<= BLOCK_CHAIN_INVENTORY(0x09),   ChainInventory
=> FETCH_INV_DATA(0x07),  Inventory [BlockID]
<= BLOCK(0x02),
<= BLOCK(0x02),
<= BLOCK(0x02),
....
依次循环，直到开始 INVENTORY 逻辑

# handshake & ping pong logic
## normal handshake
=> P2P_HELLO(0x20),
<= P2P_HELLO(0x20),
(handshake finished)

## disconnect handshake
=> P2P_HELLO(0x20),  HelloMessage
<= P2P_DISCONNECT(0x21),  DisconnectMessage

## ping & pong
=> P2P_PING(0x22),  PingMessage, FIXED_PAYLOAD = Hex.decode("C0");
<= P2P_PONG(0x23),  PongMessage, FIXED_PAYLOAD = Hex.decode("C0");
<= P2P_PING
=> P2P_PONG


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
