# Meroxidizer: A Meros Miner in Rust

## Building

Make sure you have git submodules checked out:
```
git submodule update --init
```

Install rust via https://rustup.rs/

You'll also need to install Clang

### To run once

```
RUST_LOG=info cargo run --release -- [options]
```

### To install

```
cargo install --path .
RUST_LOG=info meroxidizer [options]
```

Note that you must specify thread counts, see the next section for details.

## Options

```
meroxidizer 0.1.0

USAGE:
    meroxidizer [FLAGS] [OPTIONS] --bls-threads <bls-threads> --randomx-init-threads <randomx-init-threads> --randomx-threads <randomx-threads>

FLAGS:
    -h, --help                      Prints help information
    -o, --output-hash-rate          If the hash rate should be logged every 30 seconds
    -l, --randomx-large-pages       If large pages should be used for RandomX. Requires special configuration at the OS
                                    level
    -k, --randomx-stop-for-rekey    If mining should stop when the RandomX key changes. Advantage: doesn't double memory
                                    usage during RandomX key changes. Disadvantage: stops mining for a few seconds every
                                    other day. But mining would only progress on the old key anyways, which would be
                                    useless. This is a target for future improvement
    -V, --version                   Prints version information

OPTIONS:
    -b, --bls-threads <bls-threads>                      The number of threads to use for BLS signing
    -i, --randomx-init-threads <randomx-init-threads>
            The number of threads to use to initialize RandomX. Only matters on startup and on RandomX key change

    -t, --randomx-threads <randomx-threads>              The number of threads to use for RandomX. Must be even
    -r, --rpc <rpc>                                      The RPC address and port [default: localhost:5133]
```

This also accepts the following env variables:
- **RUST_LOG**: Set the logging level, see [env_logger documentation](https://docs.rs/env_logger/latest/env_logger/index.html) for details.
- **MEROS_MINER_KEY**: Set the Meros miner private key.
  **Warning**: the node is still somewhat trusted,
  so don't use this to run against an untrusted node.
  This is still useful if you want to run miners with multiple nodes,
  as having the same key on the nodes would cause a merit removal
  and destroy your merit.

## Example Invocation

I've split this command into multiple lines for readability,
but to actually run this you'd need to remove the comments
and merge it into one line.

```sh
RUST_LOG=info # Enable reasonable logging
cargo run --release -- # Compile and run
-r localhost:5133 # Connect to the Meros node at localhost:5133
-b 10 # Use 10 threads for BLS signing
-i 32 # Use 32 threads to initialize RandomX
-t 26 # Use 26 threads to run RandomX
-l # Enable RandomX large pages
-k # Pause mining during RandomX key rotation
```
