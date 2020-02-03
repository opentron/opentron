# Create TRC20 on Chain

./nile-wallet-cli.sh contract create TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    --abi examples/out/FixedSupplyToken.abi --code examples/out/FixedSupplyToken.bin

## ABIs

```text
function allowance(address _owner, address _spender) returns (uint256)
    => dd62ed3e: allowance(address,address)
function approve(address _spender, uint256 _amount) returns (bool)
    => 095ea7b3: approve(address,uint256)
function balanceOf(address _owner) returns (uint256)
    => 70a08231: balanceOf(address)
function decimals() returns (uint8)
    => 313ce567: decimals()
function name() returns (string)
    => 06fdde03: name()
function owner() returns (address)
    => 8da5cb5b: owner()
function symbol() returns (string)
    => 95d89b41: symbol()
function totalSupply() returns (uint256)
    => 18160ddd: totalSupply()
function transfer(address _to, uint256 _amount) returns (bool)
    => a9059cbb: transfer(address,uint256)
function transferFrom(address _from, address _to, uint256 _amount) returns (bool)
    => 23b872dd: transferFrom(address,address,uint256)
```

## transfering

```shell
./nile-wallet-cli.sh contract call TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    THi2qJf6XmvTJSpZHc17HgQsmJop6kb3ia \
    'balanceOf(address)' -- TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu

./nile-wallet-cli.sh contract call TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    THi2qJf6XmvTJSpZHc17HgQsmJop6kb3ia \
    'transfer(address,uint256)' -- TJRabPrwbZy45sbavfcjinPJC18kjpRTv8 100000

./nile-wallet-cli.sh contract call --const TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu \
    THi2qJf6XmvTJSpZHc17HgQsmJop6kb3ia 'name()'
```
