# The GraphQL API

OpenTron natively supports GraphQL, in a [EIP-1767](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1767.md) style.

Current GraphQL API of OpenTron is an experimental one, which means it might changes often.
Feel free to make a feature request or bug request.

## Get started

To enable GraphQL support, you need to check the following section in `etc/config.toml`:

```toml
[graphql]
enable = true
endpoint = "0.0.0.0:3000"
```

Open your browser with <http://localhost:3000/>, play with queries and mutations in GraphQL Playground.

## Queries

```text
apiVersion
nodeInfo: running state of OpenTron node
syncing: syncing state of OpenTron node
block: query block, almost the same API as EIP-1767
blocks
transaction
logs: log entries filter style query
account: account state query (state-db)
call: dummy execute a transaction with current block state(like constant/view/pure call).
estimateEnergy: estimate energy for a smart contract call
asset: TRC10 query
chain: chain parameter query(proposals)
```

## Mutations

```text
sendRawTransaction: TODO
```
