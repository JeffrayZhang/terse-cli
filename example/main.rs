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

#[command]
fn command_four() -> String {
    "command four".to_string()
}

subcommands!(my_subcommands, [command_two, command_three, command_four]);
subcommands!(cli, [command_one, my_subcommands]);

fn main() {
    cli::run(cli::Args::parse());
}

// you can also use `--help` as you would expect
// Example: cargo run -- my-subcommands --help
