
# Terse CLI

A wrapper around [clap](https://github.com/clap-rs/clap) that lets you build CLI applications with very little boilerplate code.

Modeled after [tiangolo's typer python library](https://github.com/fastapi/typer), you simply define your commands and subcommands as functions and annotate them with the `#[command]` attribute.

## Current Status: Alpha

This is a work in progress. The core functionality is implemented, but if you want any customization on how your CLI is used (e.g. positional arguments, custom help messages, etc.) those things are not yet supported.

Known issues:
- [ ] every command must have a return type that implements `Display`
- [ ] positional arguments are not yet supported
- [ ] argument docs are not yet supported

## Installation

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

/// Example: cargo run -- my-subcommands command-four
#[command]
fn command_four() -> String {
    "command four".to_string()
}

subcommands!(
    /// These are the subcommands
    my_subcommands, [command_two, command_three, command_four]);

subcommands!(
    /// Example docs for the "root"
    cli, [command_one, my_subcommands]);

fn main() {
    cli::run(cli::Args::parse());
}

// you can also use `--help` as you would expect
// Example: cargo run -- my-subcommands --help

```
