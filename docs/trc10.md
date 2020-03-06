# Manage TRC10 tokens

## Issuing new TRC10 token

```shell
./target/debug/wallet-cli asset issue TXBUwpDrRYfSH3MNha5amQ1SkprDBgRhpd \
    MoonCoin 1_0000_0000 \
    --abbr MOON --url https://example.com --description "The Moon Coin is a test coin" \
    --issuing-period "2020-01-09T00:00:00Z" "2020-01-10T00:00:00Z" \
    --freeze 1_0000=5 --freeze 2_0000=10 -s
```

## Transfering TRC10 token

```shell
./target/debug/wallet-cli asset transfer \
    TXBUwpDrRYfSH3MNha5amQ1SkprDBgRhpd TNSrdRdKQa1gpEyJ18okFEcwGyiEN7rtcp \
    10_000_000 "HI" --token-id 1000016
```

## Participate Issuing

When a TRC10 asset is issuing, you can participate the issuing. The following transaction
exchange ICO tokens with 1_TRX.

```shell
./target/debug/wallet-cli asset participate TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    TXBUwpDrRYfSH3MNha5amQ1SkprDBgRhpd 1_TRX --token-id 1000018
```
