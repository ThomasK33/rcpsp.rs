# RCPSP.rs

RCPSP.rs is a tabu search based local-search algorithm for the the Resource-Constrained Project Scheduling Problem in Rust.

## Installation

### Local Dev Container in VS Code

- TODO: Recommend remote code container extension

### Github Codespaces

- TODO: Describe how to open repo on Github

### Local Installation

- TODO: How to install rust

- TODO: Recommend VS Code & recommended extension in repo

## Testing

```bash
cargo test
```

## Benchmarks

```bash
cargo bench
```

## Running the scheduler binary

Running a debug build

```bash
cargo run -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p -v
```

Running a release build

```bash
cargo run --release -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p -v
```

Running the result evaluation utility

```bash
cargo run --release -- benchmark ./examples/j30.sm ./j30_results.txt
```

## Using the library

Add the following line to your `Cargo.toml`-file's `[dependencies]` section:

TODO: Change this to use the github git url

```toml
rcpsp = { path = "../rcpsp" }
```

Using the library:

```rust
// TODO: Add example of how to use the library
```

## Resources

This implementation relies on the ideas and is inspired by the following papers:

- Libor Bukata, Premysl Sucha, and Zdeněk Hanzálek. Solving the resource constrained project scheduling problem using the
parallel tabu search designed for the cuda platform. Journal of Parallel and Distributed Computing, 77, 11 2014.
