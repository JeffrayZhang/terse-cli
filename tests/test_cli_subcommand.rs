use terse_cli_lib::{command, subcommands};
use pretty_assertions::assert_eq;
use quote::quote;

#[test]
pub fn test_command_macro() {
    let in_stream = quote! {
        fn my_func(a: i32, b: i32) -> i32 {
            a + b
        }
    };
    let attr_stream = quote! { #[subcommand] };
    let out_stream = command(attr_stream, in_stream).unwrap();
    let expected = quote! {
        mod my_func {
            use super::my_func;
            use clap::{command, Parser};
            #[derive(Parser)]
            # [command (version , about , long_about = None)]
            pub struct Args {
                #[arg(long)]
                a: i32,
                #[arg(long)]
                b: i32
            }
            pub fn run(Args { a, b }: Args) {
                let result = my_func(a, b);
                println!("{}", result);
            }
        }
        fn my_func(a: i32, b: i32) -> i32 {
            a + b
        }
    };
    assert_eq!(out_stream.to_string(), expected.to_string());
}



#[test]
pub fn test_command_macro_no_args() {
    let in_stream = quote! {
        fn my_func() -> i32 {
            42
        }
    };
    let attr_stream = quote! { #[subcommand] };
    let out_stream = command(attr_stream, in_stream).unwrap();
    let expected = quote! {
        mod my_func {
            use super::my_func;
            use clap::{command, Parser};
        
            #[derive(Parser)]
            #[command(version, about, long_about = None)]
            pub struct Args {}
        
            pub fn run(Args {}: Args) {
                let result = my_func();
                println!("{}", result);
            }
        }

        fn my_func() -> i32 {
            42
        }        
    };
    assert_eq!(out_stream.to_string(), expected.to_string());
}

#[test]
pub fn test_subcommands_macro() {
    let in_stream = quote! {
        cli, [command_one, command_two]
    };
    let out_stream = subcommands(in_stream).unwrap();

    let expected = quote! {
        mod cli {
            use super::{command_one, command_two};
            use clap::{command, Parser, Subcommand};

            #[derive(Subcommand)]
            pub enum Subcommands {
                CommandOne(command_one::Args),
                CommandTwo(command_two::Args)
            }

            #[derive(Parser)]
            #[command(version, about, long_about = None)]
            pub struct Args {
                #[command(subcommand)]
                command: Subcommands,
            }

            pub fn run(Args { command }: Args) {
                match command {
                    Subcommands::CommandOne(args) => command_one::run(args),
                    Subcommands::CommandTwo(args) => command_two::run(args)
                };
            }
        }
    };
    assert_eq!(out_stream.to_string(), expected.to_string());
}
