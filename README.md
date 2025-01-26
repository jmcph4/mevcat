# `mevcat` #

`mevcat` is a lightweight tool for interating with various MEV APIs, in the spirit of [`nc(1)`](https://linux.die.net/man/1/nc).

## Installation ##

```
$ cargo install mevcat
```

## Usage ##

```
$ mevcat -h
Lightweight tool for interacting with MEV APIs

Usage: mevcat [OPTIONS] [ENDPOINT]

Arguments:
  [ENDPOINT]  Remote host to submit RPCs to

Options:
  -l, --listen <LISTEN>  Start an MEV server instance
  -p, --port <PORT>      Port number to use
  -s, --suffix <SUFFIX>  URL suffix to bind to
  -m, --method <METHOD>  RPC verb to use (client-mode only)
  -r, --raw              Send standard input to the remote host
  -h, --help             Print help
  -V, --version          Print version
```

There are two modes: client and server.

### Client Mode ###

When operating in client mode, `mevcat` accepts a JSON RPC payload from standard input and submits it to the provided RPC endpoint. The RPC method to use is specified via `-m` (the default is `eth_sendBundle`).

```
$ mevcat https://rpc.beaverbuild.org
{"txs":["0x02f8b20181948449bdee618501dcd6500083016b93942dabcea55a12d73191aece59f508b191fb68adac80b844095ea7b300000000000000000000000054e44dbb92dba848ace27f44c0cb4268981ef1cc00000000000000000000000000000000000000000000000052616e065f6915ebc080a0c497b6e53d7cb78e68c37f6186c8bb9e1b8a55c3e22462163495979b25c2caafa052769811779f438b73159c4cc6a05a889da8c1a16e432c2e37e3415c9a0b9887"],"blockNumber":"0x1361bd3"}
{"id":1,"jsonrpc":"2.0","result":{"bundleHash":"0xfc53193c7eb836aa28c45f11db07e41429b3b580c632e18ab37d1b92d66df641"}}
```

Raw mode (`-r`) will simply `POST` the entire (line-delimited) contents of standard input to the endpoint.

### Server Mode ###

In server mode, `mevcat` listens on the specified socket for incoming JSON RPC submissions (think `nc -l`). It exposes the [`EthBundleApi`](https://reth.rs/docs/reth_rpc_eth_api/bundle/trait.EthBundleApiServer.html).

```
$ RUST_LOG=info mevcat -l 0.0.0.0:3000
 2025-01-26T02:38:22.423Z INFO  mevcat > Bound to 0.0.0.0:3000
```

