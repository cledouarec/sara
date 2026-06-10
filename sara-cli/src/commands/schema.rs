//! Schema command implementation.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Args;
use sara_core::config::Config;
use sara_core::schema::{self, Schema};

/// Arguments for the schema command.
#[derive(Args, Debug)]
pub struct SchemaArgs {
    /// Export the built-in default model instead of the active one
    #[arg(long)]
    pub builtin: bool,

    /// Write the schema to a file instead of standard output
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

/// Runs the schema command.
pub fn run(args: &SchemaArgs, _config: &Config) -> Result<ExitCode, Box<dyn Error>> {
    let yaml = if args.builtin {
        Schema::builtin().to_yaml()?
    } else {
        schema::active().to_yaml()?
    };

    match &args.output {
        Some(path) => fs::write(path, yaml)?,
        None => print!("{yaml}"),
    }

    Ok(ExitCode::SUCCESS)
}
