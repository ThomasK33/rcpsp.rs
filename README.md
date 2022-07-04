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

- TODO: How to install rust

- TODO: Recommend VS Code & recommended extension in the repo

## Testing

```bash
cargo test
```

## Benchmarks

```bash
cargo bench
```

## Running the scheduler binary

To run the scheduler in debug mode, one should execute:

```bash
cargo run -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p -v
```

To run a release build of the scheduler, one should:

```bash
cargo run --release -- schedule ./examples/j30.sm/j3045_10.sm --swr 15 --tls 100 --misb 1500 --noi 3000 -p -v
```

One can evaluate the scheduler quality by running:

```bash
cargo run --release -- benchmark ./examples/j30.sm ./j30_results.txt
```

## Using the library

Add the following line to your `Cargo.toml`-file's `[dependencies]` section:

TODO: Change this to use the Github git URL

```toml
rcpsp = { path = "../rcpsp" }
```

Using the library:

```rust
// TODO: Add an example of how to use the library
```

## Resources

Inspiration was drawn from the following resources:

- Libor Bukata, Premysl Sucha, and Zdeněk Hanzálek. Solving the resource constrained project scheduling problem using the
parallel tabu search designed for the cuda platform. Journal of Parallel and Distributed Computing, 77, 11 2014.
