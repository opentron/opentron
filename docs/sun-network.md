# Sun-Network Playground

## Special Signature

<https://github.com/tronprotocol/sun-network/blob/d05d273e05f4581c45f40df001fb489c21cda1d7/js-sdk/src/index.js#L39>

vs

<https://github.com/TRON-US/tronweb/blob/master/src/utils/crypto.js#L50>

```js
let byteArr = this.utils.code.hexStr2byteArray(transaction.txID).concat(chainIdByteArr);
let byteArrHash = this.sidechain.utils.ethersUtils.sha256(byteArr);
```

## Deposit

```console
> wallet-cli --network testnet contract call TBvJUBXorwBPzqvV38vjDgegj5Eh6g2Tsq \
    TFLtPoEtVJBMcj6kZPrQrwEdM3W3shxsBU 'depositTRX()' --value 10_TRX
```

Then you got coresponding coin on sidechain:

```console
> wallet-cli --network dappchain-testnet  get account TBvJUBXorwBPzqvV38vjDgegj5Eh6g2Tsq
"balance": 10_000000
```
