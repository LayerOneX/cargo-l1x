use cargo_l1x::{build::build, create::create};
use colored::Colorize;

use anyhow::{anyhow, Result};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(long_about = None, bin_name = "cargo l1x create")]
struct CreateCli {
    #[arg(help = "The name of the contract to create")]
    name: String,
    #[arg(
        short,
        long,
        default_value = "local_default",
        help = "The template to use when creating the contract (default/ft/nft). Templates from https://github.com/L1X-Foundation/cargo-l1x-templates are used"
    )]
    template: String,
}

fn get_command(args: &mut Vec<String>) -> String {
    let command = if let Some(command) = args.get(0) {
        if command == "l1x" {
            if let Some(command) = args.get(1) {
                let command = command.clone();
                *args = args[1..].to_vec();
                command.to_string()
            } else {
                "help".to_string()
            }
        } else {
            *args = args[1..].to_vec();
            get_command(args)
        }
    } else {
        "help".to_string()
    };
    command
}

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    let command = get_command(&mut args);

    match command.as_str() {
        "help" | "--help" | "-h" => {
            display_help(
                "cargo l1x <COMMAND>",
                vec![
                "build [OPTIONS]          Build the contract. See `cargo l1x build --help` for more information.",
                "create <NAME> [OPTIONS]  Create a new contract. See `cargo l1x create --help` for more information."
                ],
                vec![
                "-h, --help               Display this help message",
                "-V, --version            Display version information",
                ],
                vec![]
            );
        }
        "--version" | "-V" => {
            println!("cargo-l1x {}", env!("CARGO_PKG_VERSION"));
        }
        "build" => {
            let args = args[1..].to_vec();
            if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
                display_help(
                        "cargo l1x build [OPTIONS] [CARGO_OPTIONS]",
                        vec![],
                        vec![
                            "-h, --help               Display this help message",
                            "--no-strip               Do not strip debug information and symbols from the contract binary (useful for debugging)",
                            "CARGO_OPTIONS            Any optsions that can be passed to `cargo build` can be passed here. See `cargo build --help` for more information. There is some exceptions:",
                            "                         --target | --message-format | --version | --manifest-path | --profile",
                        ],
                        vec![
                            "LLVM_BIN_PATH            The path to 'bin' directory where 'llc' is stored. Useful if 'llc' is not present in any directory in PATH"     
                        ],
                    );
                return Ok(());
            }
            let target_dir = cargo_metadata::MetadataCommand::new()
                .exec()
                .expect("cargo metadata failed")
                .target_directory;
            check_args_not_contains(
                args.to_vec(),
                vec![
                    "--target",
                    "--message-format",
                    "--version",
                    "--manifest-path",
                    "--profile",
                ],
            )?;

            println!("Building contracts...");
            build(args.to_vec(), target_dir.into())?;

            println!("🎉 Compilation and processing completed!");
        }
        "create" => {
            let CreateCli { name, template } = CreateCli::parse_from(args);

            create(name, template.clone())?;

            println!("🎉 The contract was generated from '{}' template", template);
        }
        e => {
            println!("Unknown command: {e}");
        }
    };
    return Ok(());
}

fn check_args_not_contains(args: Vec<String>, exclude: Vec<&str>) -> Result<()> {
    for arg in args {
        for e in &exclude {
            if arg.starts_with(e) {
                return Err(anyhow!("This argument cannot be changed: {}", e));
            }
        }
    }
    Ok(())
}

fn display_help(
    usage: &str,
    arguments: Vec<&str>,
    options: Vec<&str>,
    environment_vars: Vec<&str>,
) {
    println!("{}: {}", "Usage".underline(), usage);
    if !arguments.is_empty() {
        println!("\n{}:", "Arguments".underline());
        for arg in arguments {
            println!("  {}", arg);
        }
    }
    if !options.is_empty() {
        println!("\n{}:", "Options".underline());
        for opt in options {
            println!("  {}", opt);
        }
    }
    if !environment_vars.is_empty() {
        println!("\n{}:", "Environment variables".underline());
        for var in environment_vars {
            println!("  {}", var);
        }
    }
}
