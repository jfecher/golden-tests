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

    #[clap(help = "The directory to search for test files recursively within, or a single file to test")]
    test_path: PathBuf,

    #[clap(
        help = "Prefix string for test commands. This is usually the same as the comment syntax in the language you are testing. For example, in C this would be '// '"
    )]
    test_prefix: String,

    #[clap(
        long,
        default_value = "args:",
        help = "Prefix string for the command line arguments to be passed to the command, before the program file path."
    )]
    args_prefix: String,

    #[clap(
        long,
        default_value = "args after:",
        help = "Prefix string for the command line arguments to be passed to the command, after the program file path."
    )]
    args_after_prefix: String,

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
        help = "Update the expected output of each test file to match the actual output"
    )]
    overwrite: bool,

    #[clap(long, default_value = "", help = "Arguments to add before the file name when running every test file")]
    base_args: String,

    #[clap(long, default_value = "", help = "Arguments to add after the file name when running every test file")]
    base_args_after: String,
}

fn main() {
    let args = Args::parse();

    let config = TestConfig {
        binary_path: args.binary_path,
        test_path: args.test_path,
        test_line_prefix: args.test_prefix,
        test_args_prefix: args.args_prefix,
        test_args_after_prefix: args.args_after_prefix,
        test_stdout_prefix: args.stdout_prefix,
        test_stderr_prefix: args.stderr_prefix,
        test_exit_status_prefix: args.exit_status_prefix,
        overwrite_tests: args.overwrite,
        base_args: args.base_args,
        base_args_after: args.base_args_after,
    };

    config.run_tests().unwrap_or_else(|_| std::process::exit(1));
}
