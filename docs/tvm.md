# TVM

## Stages

### AllowTvm = 9

Version: 2.1

### AllowTvmTransferTrc10Upgrade = 18

Version: 3.2

CALLTOKEN, TOKENBALANCE, CALLTOKENVALUE, CALLTOKENID

### AllowMultisig = 20

Version: 3.5

- Add `isTokenTransferMsg = true` property to token call. Check token id
- Fix `ORIGIN` opcode, 21-bytes to 20-byte

NOTE: This TVM change is under multisig scope. Might be useless.

### AllowTvmConstantinopleUpgrade = 26

Version: 3.6.0

- ClearABI builtin contract
- reject delegate resource to contract address
- Save runtime code from execution result of deploy code
- OpCode: SHL, SHR, SAR, EXTCODEHASH, fake and buggy CREATE2
- Introduce TransferException

### AllowTvmSolidity059Upgrade = 32

Version: 3.6.5

- create account while transfer TRX/TRC10
- Precompile: batchvalidatesign, validatemultisign
- OpCode: ISCONTRACT

### AllowTvmShieldedUpgrade = 39

Version: 4.0.0 / 4.0.1

- Precompile: verifyMintProof, verifyTransferProof, verifyBurnProof, pedersenHash

NOTE: `pedersenHash` is called `merkleTree` in java-tron, which is inconsistent.

### AllowTvmIstanbulUpgrade

Version: 4.1.0

- Impl EVM OpCode: CHAINID, SELFBALANCE
- OpCode Change:
  - CREATE2 impl
- Precompile Change:
  - bn128add: energy change from 500 to 150
  - bn128mul: energy change from 40000 to 6000
  - bn128pairing: energy change
