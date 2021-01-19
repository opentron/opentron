# proto

Refactored version of core protobuf files.

## Key points

- Each file is in its own scope
  - `chain.proto`: data on the "real blockchain", blocks and transactions
  - `discovery.proto`: peer discovery protocol of udp/18888
  - `channel.proto`: sync and relay protocol of tcp/18888
  - `contract.proto`: all system contracts
  - `state.proto`: the state-db part
- Handle "unknown fields" via `oneof`
  - a desgin fault, and is still vulnerable now
