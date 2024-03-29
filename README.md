# RCPSP.rs

RCPSP.rs is a tabu search-based local-search algorithm for the Resource-Constrained Project Scheduling Problem in Rust.

## Installation

### Local Dev Container in VS Code

To run this repo inside a dev container, please install the corresponding [Remote Code Extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) in VS Code and open the repo inside it.
A corresponding dev container manifest is already defined.

Please refer to the [official documentation](https://code.visualstudio.com/docs/remote/containers) for more extensive and detailed instructions.

### Github Codespaces

As this project configures a dev container, running a Github Codespace is easily possible.

To open a cloud-hosted development environment, select the green code button, select "Codespaces," and then select "Create codespace on main."

This operation will take a couple of minutes, after which one is ready to go.

### Local Installation

Follow the [official Rust language's getting started guide](https://www.rust-lang.org/learn/get-started) to set up Rust on your local machine.

## Testing

One can execute a test suite by running:

```bash
cargo test
```

## Graph Visualization

To visualize the PSP Lib problem as a graph, one needs to have previously installed [graphviz](https://graphviz.org/).

One can generate a graph using:

```bash
cargo run -- graph ./examples/file_name.sm ./file_name.dot
dot -Tpng file_name.dot > file_name.png
```

## Benchmarks

One can perform Criterion benchmarks by executing:

```bash
cargo bench
```

The location of the generated report is: `target/criterion/report/index.html`.

## Running the scheduler binary

To run the scheduler in debug mode, without LLVM optimizations applied, one should execute:

```bash
cargo run -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p -v
```

To run a release build of the scheduler, with LLVM optimizations on, one should:

```bash
cargo run --release -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p
```

To run a release build of the scheduler using a rayon-based multi-schedule scheduling algorithm, with LLVM optimizations on, one should:

```bash
cargo run --release -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p --algo rayon-multi
```

To run a release build of the scheduler using a custom multi-threaded scheduling algorithm, with LLVM optimizations on, one should:

```bash
cargo run --release -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p --algo custom
```

One can evaluate the scheduler quality by running:

```bash
cargo run --release -- benchmark ./examples/j30.sm ./j30_results.csv
cargo run --release -- benchmark ./examples/j30.sm ./j30_results_rayon_multi.csv --algo rayon-multi
cargo run --release -- benchmark ./examples/j30.sm ./j30_results_custom.csv --algo custom
```

<!-- ## Using the library

Add the following line to your `Cargo.toml`-file's `[dependencies]` section:

TODO: Change this to use the Github git URL

```toml
rcpsp = { path = "../rcpsp" }
```

Using the library:

```rust
// TODO: Add an example of how to use the library
``` -->

## Resources

Inspiration was drawn from the following resources:

- Libor Bukata, Premysl Sucha, and Zdeněk Hanzálek. Solving the resource constrained project scheduling problem using the
parallel tabu search designed for the cuda platform. Journal of Parallel and Distributed Computing, 77, 11 2014.
