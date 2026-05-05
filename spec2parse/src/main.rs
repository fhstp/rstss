use clap::{Parser, Subcommand};
use std::{env, process::exit};

mod files;
mod fonts;
mod helpers;
mod spec;
mod table;
mod xml;
mod cmd_version;
mod cmd_generate;
mod cmd_parse;

#[derive(Debug, Subcommand)]
#[command(help_template = "{all-args}", disable_help_subcommand=true)]
enum SubCommands {
    #[command(name = "generate", alias = "gen", about = "generate code fragments from parsed spec (json file)")]
    Generate(cmd_generate::Param),
    #[command(name = "parse", alias = "par", about = "parse specification from fodg pages to json file")]
    Parse(cmd_parse::Param),
    #[command(name = "version", alias = "ver", about = "build information", disable_help_flag = true)]
    Version(cmd_version::Param),
}

#[derive(Debug, Parser)]
#[command(help_template = "{subcommands}")]
struct Cmd {
    #[command(subcommand)]
    command: SubCommands,
}


fn main() {
    if env::args().len() == 1 {
        println!("TPM v2 specification parser and code fragments generator");
        println!("...do what? (-h or --help for help)");
        exit(0);
    }

    let parsed = Cmd::parse();
    match parsed.command {
        SubCommands::Generate(param) => cmd_generate::run(param),
        SubCommands::Parse(param) => cmd_parse::run(param),
        SubCommands::Version(param) => cmd_version::run(param),
    }
}
