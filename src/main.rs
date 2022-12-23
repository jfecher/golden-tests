mod config;
mod diff_printer;
mod error;
mod runner;

use crate::config::TestConfig;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(help = "The program to run for each test file")]
    binary_path: PathBuf,

    #[clap(help = "The directory to search for test files recursively within")]
    test_directory: PathBuf,

    #[clap(
        help = "Prefix string for test commands. This is usually the same as the comment syntax in the language you are testing. For example, in C this would be '// '"
    )]
    test_prefix: String,

    #[clap(long, default_value = "args:", help = "The program to run for each test file")]
    args_prefix: String,

    #[clap(
        long,
        default_value = "expected stdout:",
        help = "The program to run for each test file"
    )]
    stdout_prefix: String,

    #[clap(
        long,
        default_value = "expected stderr:",
        help = "The program to run for each test file"
    )]
    stderr_prefix: String,

    #[clap(
        long,
        default_value = "expected exit status:",
        help = "The program to run for each test file"
    )]
    exit_status_prefix: String,

    #[clap(
        long,
        help = "Accept what the current output, update the files to match this"
    )]
    overwrite: bool,
}

fn main() {
    let args = Args::parse();

    let config = match TestConfig::with_custom_keywords(
        args.binary_path,
        args.test_directory,
        &args.test_prefix,
        &args.args_prefix,
        &args.stdout_prefix,
        &args.stderr_prefix,
        &args.exit_status_prefix,
        args.overwrite,
    ) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {}", error);
            return;
        }
    };

    config.run_tests().unwrap_or_else(|_| std::process::exit(1));
}
