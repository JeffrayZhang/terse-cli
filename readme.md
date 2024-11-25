
# Terse CLI

A wrapper around [clap](https://github.com/clap-rs/clap) that lets you build CLI applications with very little boilerplate code.

Modeled after [tiangolo's typer python library](https://github.com/fastapi/typer), you simply define your commands and subcommands as functions and annotate them with the `#[command]` attribute.


## Installation

Make sure you're using nightly rust:

```sh
$ cat > rust-toolchain.toml <<EOF
[toolchain]
channel = "nightly"
EOF
```

Install both clap and terse_cli:

```sh
$ cargo add clap --features derive
$ cargo add terse_cli
```

## Example

Below snippet is from [./example/main.rs](./example/main.rs).

```rs
use clap::Parser;
use terse_cli::{command, subcommands};

/// Example: cargo run -- command-one --a 3
#[command]
fn command_one(a: i32, b: Option<i32>) -> i32 {
    a + b.unwrap_or(0)
}

/// Example: cargo run -- my-subcommands command-two --name Bob
#[command]
fn command_two(name: String) -> String {
    format!("hello {}", name)
}

/// Example: cargo run -- my-subcommands command-three --a 7 --b 3
#[command]
fn command_three(a: i32, b: i32) -> String {
    format!("the difference is {}", a - b)
}

subcommands!(my_subcommands, [command_two, command_three]);
subcommands!(cli, [command_one, my_subcommands]);

fn main() {
    cli::run(cli::Args::parse());
}

// you can also use `--help` as you would expect
// Example: cargo run -- my-subcommands --help
```


#### Current Status: Alpha

This is a work in progress. The core functionality is implemented, but if you want any customization on how your CLI is used (e.g. positional arguments, custom help messages, etc.) those things are not yet supported.


