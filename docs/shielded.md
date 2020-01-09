# Shielded Transaction

```protobuf
rpc GetNewShieldedAddress (EmptyMessage) returns (ShieldedAddressInfo) {}

rpc GetSpendingKey (EmptyMessage) returns (BytesMessage) {}
rpc GetDiversifier (EmptyMessage) returns (DiversifierMessage) {}

rpc GetExpandedSpendingKey (BytesMessage) returns (ExpandedSpendingKeyMessage) {}
rpc GetAkFromAsk (BytesMessage) returns (BytesMessage) {}
rpc GetNkFromNsk (BytesMessage) returns (BytesMessage) {}
rpc GetIncomingViewingKey (ViewingKeyMessage) returns (IncomingViewingKeyMessage) {}
rpc GetZenPaymentAddress (IncomingViewingKeyDiversifierMessage) returns (PaymentAddressMessage) {}

rpc GetRcm (EmptyMessage) returns (BytesMessage) {}
rpc GetMerkleTreeVoucherInfo (OutputPointInfo) returns (IncrementalMerkleVoucherInfo) {}
rpc CreateShieldedTransaction (PrivateParameters) returns (TransactionExtention) {};

rpc ScanNoteByIvk (IvkDecryptParameters) returns (DecryptNotes) {};
rpc ScanAndMarkNoteByIvk (IvkDecryptAndMarkParameters) returns (DecryptNotesMarked) {};
rpc ScanNoteByOvk (OvkDecryptParameters) returns (DecryptNotes) {};
rpc IsSpend (NoteParameters) returns (SpendResult) {}

rpc CreateShieldedTransactionWithoutSpendAuthSig (PrivateParametersWithoutAsk) returns (TransactionExtention) {};

rpc GetShieldTransactionHash (Transaction) returns (BytesMessage) {};

rpc CreateSpendAuthSig (SpendAuthSigParameters) returns (BytesMessage) {};

rpc CreateShieldNullifier (NfParameters) returns (BytesMessage) {};
```
