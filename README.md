# Negentropy

## Description

Rust implementation of the [negentropy](https://github.com/hoytech/negentropy) set-reconciliation protocol.

## Minimum Supported Rust Version (MSRV)

These crates are built with the Rust language version `2018` and require a minimum compiler version of `1.51.0`

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## Flame Graph and perf

Install [flamegraph](https://github.com/flamegraph-rs/flamegraph) and then run `just graph`. 
You'll find a new file in the project root called `flamegraph.svg`: open it in a browser.

In the terminal you should see something like:

```bash
Client init took 0 ms
Relay items: 1000000
Relay reconcile took 25 ms
Client reconcile took 39 ms
[ perf record: Woken up 10 times to write data ]
[ perf record: Captured and wrote 2.406 MB perf.data (150 samples) ]
```

## Benchmarks (unstable)

To run the benchmarks use: `just bench`

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details
