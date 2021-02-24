# Txpool - How transaction is handled

## FullNode.java

Application

```
tronNetService.start();
consensusService.start();
MetricsUtil.init();
appT.addService(rpcApiService);
appT.addService(httpApiService);
appT.addService(rpcApiServiceOnSolidity);
appT.addService(httpApiOnSolidityService);
appT.addService(rpcApiServiceOnPBFT);
appT.addService(httpApiOnPBFTService);
```

### tronNetService

```
channelManager
advService
syncService
peerStatusCheck

message handlers(channel message):
SYNC_BLOCK_CHAIN
BLOCK_CHAIN_INVENTORY
INVENTORY => Adv(hashes)
 - Transaction
 - Block
FETCH_INV_DATA
 - Transaction
 - Block
BLOCK
TRXS
PBFT_COMMIT_MSG
```

### rpcApiService(broadcastTransaction)

```
wallet.broadcastTransaction(req)
Wallet(broadcastTransaction)

steps:

- check minEffectiveConnection
- check tooManyPending (maxTransactionPendingSize = 2000)
- check already exists (in ids cache)
- dbManager.pushTransaction(trx)
  - processTransaction
  - ? push and throws many types of errors
    - signature validation
    - execute error
    - resouce check
    - dup transaction
    - tapos check
    - expiration check
    - ...
- tronNetService.broadcast(message)
  - advService.broadcast
```

### consensusService

The miners.

```
getBlockProducedTimeOut = 50%.

BlockHandle.produce(Miner miner, long blockTime, long timeout)
manager.generateBlock(miner, blockTime, timeout)

Consensus.start()
- DposService.start()
  - dposTask.init()
    - calling produceBlock
      - manager.generateBlock
```

## Block Produce

``Manager.generateBlock(Miner miner, long blockTime, long timeout)``

- Generate block header
- iterater over pendingTransactions and rePushTransactions
  - processTransaction