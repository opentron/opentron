# Shielded TRC20

ver: 20200703

匿名 TRC20 币使用.

[白皮书](https://www.tronz.io/Shielded%20Transaction%20Protocol.pdf).

## 基础知识

z-addr 地址. 即匿名地址, 该地址通过一个 `sk` 和 一个 `d` 唯一确定.
可以公布你的 `z-addr` 让别人转账给你, 其他的各种 key 都用来做某种操作. `d` 参数往往用来形成 HD 钱包, 可以认为是地址序列.
具体如下.

```text
// sk: spending key => ask, nsk, ovk
// ask: spend authorizing key, 256 => ak
// nsk: proof authorizing key, 256 => nk
// ovk: outgoing viewing key, 256
// ivk: incoming viewing key, 256 => pkD
// d: diversifier, 11bytes
// pkD: the public key of the address, g_d^ivk
// pkD + d => z-addr
```

例如: `ztron1m445gx74mjuuyhkyru5hrx886jszfga4a7dk3mg4uarrl0cru649jz4928tm6rqul2pg645hqv5`

该地址格式为 [bech32 格式](https://en.bitcoin.it/wiki/Bech32), `ztron1` 为固定前缀. 后为编码的 `pkD` 和 `d` 信息.

地址和 `key` 相关 API 如下:

```text
wallet/getspendingkey
> generating sk

wallet/getexpandedspendingkey
> sk => aks, nsk, ovk

wallet/getakfromask
> ask => ak

wallet/getnkfromnsk
> nsk => nk

wallet/getincomingviewingkey
> ak, nk => ivk

wallet/getdiversifier
> generating d

wallet/getzenpaymentaddress
> ivk, d => z-addr, pkD
```

是不是绕成傻逼了? 既然绕成傻逼, 目前还有个老 API 可用, 一次生成所有, 少了很多痛苦.

```console
> curl https://api.nileex.io/wallet/getnewshieldedaddress
{
  "sk": "03a7e42eb715dc5523469408704fc3904db38bf3a5e5232aa22b71704fec2d3d",
  "ask": "c6ca0277c7907e462c977a4ded4bfd35d779f055bf61e141bd47713dc5074e0b",
  "nsk": "afc20d0dd9a14b41c6bc42b7431e82c568033045f1e5b50655894ae18ad62306",
  "ovk": "d89330097fb5f634f26798dc662f7ec9733ccfdfe6c0911baa580f75476d6d84",
  "ak": "c682baefe8638002c0ba3fcb58008b18bd89a4319b4598962127a2b411e7d134",
  "nk": "458dc9d273f890853b0ca1c47ebca39f429e70a68182fe676ed083ea8b5408a4",
  "ivk": "847e474d8d662ab52e16ba8af16b0ff4105ed4807b868a1c7c35f40b6bb76400",
  "d": "82a054852a62bf169f6812",
  "pkD": "7cd04d215d87fd93366741f751ff0a4ed11cd66a037970dd242f5bbdccdfce01",
  "payment_address": "ztron1s2s9fpf2v2l3d8mgzf7dqnfptkrlmyekvaqlw50lpf8dz8xkdgphjuxaysh4h0wvml8qzjzrv36"
}
```

有了匿名的 z 地址, 引入重要的三大任务:

- 向 z 地址转账, 即铸币操作 `mint`
- z 地址之间互相转账, 即匿名转账操作 `transfer`
- z 地址向传统 T 地址转账, 即销毁匿名币 `burn`

对于匿名 TRC20 交易, 整个系统采用了类似 BTC 的 utxo 模型, 而非 TRON 原本的 account balance 模型.
即通过上一笔交易的收据花掉到帐的钱(找零模型). 一笔零钱大概可以认为是 `(交易id, 地址, 金额, 交易备注, rcm)` 的元组,
其中 rcm 是 commitment randomness, 一个提前生成的随机值.

## 匿名 TRC20 合约

合约地址 <https://raw.githubusercontent.com/tronprotocol/java-tron/feature/shieldedUSDT/deploy/ShieldedTRC20.sol>

一个匿名 TRC20 合约, 必须绑定到一个已有的 TRC20 合约上. 如下是关键函数列表:

```text
constructor (address trc20ContractAddress, uint256 scalingFactorExponent)

function burn(bytes32[10] input, bytes32[2] spendAuthoritySignature, uint256 rawValue, bytes32[2] bindingSignature, address payTo, bytes32[3] c)
    => burn(bytes32[10],bytes32[2],uint256,bytes32[2],address,bytes32[3]) [4d013fde]

function mint(uint256 rawValue, bytes32[9] output, bytes32[2] bindingSignature, bytes32[21] c)
    => mint(uint256,bytes32[9],bytes32[2],bytes32[21]) [855d175e]

function transfer(bytes32[10][] input, bytes32[2][] spendAuthoritySignature, bytes32[9][] output, bytes32[2] bindingSignature, bytes32[21][] c)
    => transfer(bytes32[10][],bytes32[2][],bytes32[9][],bytes32[2],bytes32[21][]) [9110a55b]

function scalingFactor() view returns (uint256)
    => scalingFactor() [ed3437f8]
function getPath(uint256 position) view returns (bytes32, bytes32[32])
    => getPath(uint256) [e1765073]
```

基于 TRON 对该套 API 的封装, 实际使用不需要关注 `burn`, `mint`, `transfer` 函数的参数, 只需通过函数签名和工具 API 返回的字节串调用合约即可.

其中合约的构造函数传递了对应的 TRC20 合约地址, 以及一个 `scalingFactorExponent`, 用于表示匿名币到原 TRC20 的转换比例,
这个变量是 10 的指数(不是对数), 即: 传入 1, 则匿名币数额是 TRC20 的 1/10. 传入 0, 则是 1 比 1.

且该合约没有暴露公开的 TRC20 合约地址, 缺失重要必要信息. 目测是设计缺陷. 不过可以通过读取合约创建交易的末尾字节获得.

## 使用流程

第一步, 部署合约, 调用合约构造函数完成初始化

这里使用 Nile Testnet 的 JST TRC20 token `TF17BgPaZYbz8oxbjhriubPDsA7ArKoLX3`.

对应的匿名 TRC20 合约地址是 `TEkQTDyZmbY6hngxdAsxzxy9r3bUNhRjdS`. 构造函数参数为
`(TF17BgPaZYbz8oxbjhriubPDsA7ArKoLX3, 18)`. 所以 `scalingFactor` 是 10 ** 18.

Solidity 编译器需要使用 https://github.com/tronprotocol/solidity 的 `develop` 分支.

然后我们的初始地址 `TJRabPrwbZy45sbavfcjinPJC18kjpRTv8` 内有一堆代币. (通过 Nile Faucet 地址获取)

### 给 z 地址转账 - 铸币 mint

转账前有一个重要操作, 使用 TRC20 的 `approve` 转账方式, 给匿名合约地址授权.

=> transferFrom(from: addr, to: addr, uint265)

```text
./nile-wallet-cli.sh contract call TJRabPrwbZy45sbavfcjinPJC18kjpRTv8 TEVN8mXfumcy2v3xGzjELSWMPRgBWZUhbB \
    'approve(address,uint256)' --
    TGbsfpmaPuSqQyEgieQTd5aZN9XvEMga7e 1000000
```

授权 1000000 单位 TRC20 代币, 这样匿名 TRC20 合约就有权限使用 `transferFrom` 方式转账.

假设我们的目标地址是 `ztron1s2s9fpf2v2l3d8mgzf7dqnfptkrlmyekvaqlw50lpf8dz8xkdgphjuxaysh4h0wvml8qzjzrv36`.

先创建 `rcm` (这个在文档没有, 该值是曲线上的一点, 必须通过 API 或离线算法生成):

```console
> curl https://api.nileex.io/wallet/getrcm
{"value": "720c84c8b41b3dcfcc1d5997e196e6de99f07aefb9274e285a23c5599ea2c40a"}
```

随后调用合约参数构造 API `wallet/createshieldedcontractparameters`:

```py
{
  'from_amount': '10000',
  'shielded_receives': {'note': {
      'value': 1000,
      'payment_address': 'ztron1s2s9fpf2v2l3d8mgzf7dqnfptkrlmyekvaqlw50lpf8dz8xkdgphjuxaysh4h0wvml8qzjzrv36',
      'rcm': '720c84c8b41b3dcfcc1d5997e196e6de99f07aefb9274e285a23c5599ea2c40a',
      'memo': 'HEX'}},
  'shielded_TRC20_contract_address': '4148c0020ff778c4090bf196e39d51b92bf5a647b1'
}
```

- `ovk` 在早期匿名币实现里
  - `wallet-cli` 内写死变量为 `030c8c2bc59fb3eb8afb047a8ea4b028743d23e7d38c6fa30908358431e2314d`
  - 官方文档里是 `1797de3b7f33cafffe3fe18c6b43ec6760add2ad81b10978d1fca5290497ede9`, 没有写明来源
  - 实际上不填也可, 不清楚官方文档在这里增加 `ovk` 参数的用意是什么
- 其中 `from_amount` 只能是字符串型(傻逼设计)
- `note` 内 `value` 是 `from_amount` 除以 `scalingFactor`

得到一坨参数, 其中最重要的是 `trigger_contract_input`

```py
{'binding_signature': '......',
 'message_hash': '.....',
 'parameter_type': 'mint',
 'receive_description': [{'c_enc': '..........',
                          'c_out': '.....',
                          'epk': '...',
                          'note_commitment': '.....',
                          'value_commitment': '.....',
                          'zkproof': '.....'}],
 'trigger_contract_input': '.....'
}
```

`[855d175e]`(mint 函数的签名) 拼接 trigger_contract_input, 得到调用合约 `mint` 函数的最终参数, 使用发送地址调用合约并签名广播即可.

交易 <https://nile.tronscan.org/#/transaction/8483de7f10b8db4e678334d0b6a102c4280585c086b191d4320b98c2c9e6cadc>

### 查询转入转账

使用 `wallet/scanshieldedtrc20notesbyivk` API.

提供 z-addr 的 ivk, ak, nk 返回 Note 列表. 一次只能扫描 1000 个区块.

返回的 Note 会附带 `is_spent` 信息.

### z 地址之间相互转账 - transfer

转账基于 Note 到 Note, 可以是 1-2 个 Note 转入 1-2 个 Note, 金额必须平衡，使用找零机制。

需要注意的是 `alpha`, `rcm` 均为随机数。

例子 txid: f0274768b212cf1729a137926980315be04ff46d265416183d85a97bccc68b8b

### 查询转出转账

使用 `wallet/scanshieldedtrc20notesbyovk` API

提供 z-addr 的 ovk.

返回的 Note 不带 `is_spent` 字段, 理论上都是 `is_spent` = True.

### 查询 Note 是否被花掉

使用 `wallet/isshieldedtrc20contractNoteSpent` API. 提供 `ak`, `nk`.

### z 地址转入到透明地址 - burn

操作同 transfer.
