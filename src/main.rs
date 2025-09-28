mod config;
mod config_file;
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
        default_value = "",
        help = "Arguments to add before the file name when running every test file"
    )]
    base_args: String,

    #[clap(
        long,
        default_value = "",
        help = "Arguments to add after the file name when running every test file"
    )]
    base_args_after: String,

    #[clap(flatten)]
    cli_args: CliOnlyArgs,

    #[cfg(feature = "parallel")]
    #[clap(long, help = "Number of max. parallel jobs")]
    jobs: Option<usize>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliOnlyArgs {
    #[clap(
        long,
        help = "Update the expected output of each test file to match the actual output"
    )]
    overwrite: bool,
}

fn main() {
    let mut config = match config_file::read_config_file(None) {
        Some(mut config) => {
            let args = CliOnlyArgs::parse();
            config.overwrite_tests = args.overwrite;
            config
        }
        None => {
            let args = Args::parse();

            #[cfg(feature = "parallel")]
            if let Some(max_jobs) = args.jobs {
                rayon::ThreadPoolBuilder::new().num_threads(max_jobs).build_global().unwrap();
            }

            Args::parse().into_test_config()
        }
    };

    let test_line_prefix = config.test_line_prefix.to_string();
    let prefixed = |s| format!("{}{}", test_line_prefix, s);
    config.test_args_prefix = prefixed(config.test_args_prefix);
    config.test_args_after_prefix = prefixed(config.test_args_after_prefix);
    config.test_stdout_prefix = prefixed(config.test_stdout_prefix);
    config.test_stderr_prefix = prefixed(config.test_stderr_prefix);
    config.test_exit_status_prefix = prefixed(config.test_exit_status_prefix);

    config.run_tests().unwrap_or_else(|_| std::process::exit(1));
}

impl Args {
    fn into_test_config(self) -> TestConfig {
        TestConfig {
            binary_path: self.binary_path,
            test_path: self.test_path,
            test_line_prefix: self.test_prefix,
            test_args_prefix: self.args_prefix,
            test_args_after_prefix: self.args_after_prefix,
            test_stdout_prefix: self.stdout_prefix,
            test_stderr_prefix: self.stderr_prefix,
            test_exit_status_prefix: self.exit_status_prefix,
            overwrite_tests: self.cli_args.overwrite,
            base_args: self.base_args,
            base_args_after: self.base_args_after,
        }
    }
}
