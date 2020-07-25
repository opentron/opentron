# Wallet-Cli

## Smart Contract

```console
> # compile contract
> solc -o . --abi --bin Contract.sol

> # create contract
> ./target/debug/wallet-cli contract create TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
>     --abi ./Contract.abi --code ./Contract.bin \
>     --name YourSuperCoolContractName \
>     --energy-limit 1000  # must set on mainnet
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

## TRC10

### Issuing new TRC10 token

```shell
./target/debug/wallet-cli asset issue TXBUwpDrRYfSH3MNha5amQ1SkprDBgRhpd \
    MoonCoin 1_0000_0000 \
    --abbr MOON --url https://example.com --description "The Moon Coin is a test coin" \
    --issuing-period "2020-01-09T00:00:00Z" "2020-01-10T00:00:00Z" \
    --freeze 1_0000=5 --freeze 2_0000=10 -s
```

### Transfering TRC10 token

```shell
./target/debug/wallet-cli asset transfer \
    TXBUwpDrRYfSH3MNha5amQ1SkprDBgRhpd TNSrdRdKQa1gpEyJ18okFEcwGyiEN7rtcp \
    10_000_000 "HI" --token-id 1000016
```

### Participating Issuing

When a TRC10 asset is issuing, you can participate the issuing. The following transaction
exchange ICO tokens with 1_TRX.

```shell
./target/debug/wallet-cli asset participate TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    TXBUwpDrRYfSH3MNha5amQ1SkprDBgRhpd 1_TRX --token-id 1000018
```

## TRC20

### Creating a TRC20 Contract on Chain

```shell
./target/debug/wallet-cli contract create TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    --abi examples/out/FixedSupplyToken.abi --code examples/out/FixedSupplyToken.bin \
    --fee-limit 5_TRX
```

## Transfering TRC20

```shell
./target/debug/wallet-cli contract call --const TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    THi2qJf6XmvTJSpZHc17HgQsmJop6kb3ia 'name()'

./target/debug/wallet-cli contract call TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    THi2qJf6XmvTJSpZHc17HgQsmJop6kb3ia \
    'balanceOf(address)' -- TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu

./target/debug/wallet-cli contract call TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    THi2qJf6XmvTJSpZHc17HgQsmJop6kb3ia \
    'transfer(address,uint256)' -- TJRabPrwbZy45sbavfcjinPJC18kjpRTv8 100000
```
