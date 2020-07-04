#![feature(str_strip)]

//! A testing library utilizing golden tests.
//!
//! ### Why golden tests?
//!
//! Golden tests allow you to specify the output of
//! some command within a file and automatically ensure
//! that that output doesn't change. If it does, goldentests
//! will show an error-diff showing the expected and actual
//! output. This way, whenever the output of something changes
//! a human can see the change and decide if it should be kept
//! or is a bug and should be reverted.
//!
//! ### What are golden tests useful for?
//!
//! Golden tests are especially useful for applications that
//! take a file as input and produce output of some kind. For
//! example: compilers and config-parsers (well, parsers in general)
//! are two such applications that can benefit form automated golden
//! tests. In the case of a config parser, you would be able to
//! provide many config examples as tests and ensure that your
//! parser was able to read the files with the expected stdout/stderr
//! output and exit code.
//!
//! ### How do I get started?
//!
//! Include a test in your program that looks something like this:
//!
//! ```rust
//! use goldentests::{ TestConfig, TestResult };
//! 
//! #[test]
//! fn run_goldentests() -> TestResult<()> {
//!     // Replace "// " with your language's/parser's comment syntax.
//!     // This tells golden tests to embed its keywords in lines beginning with "// "
//!     let config = TestConfig::new("target/debug/my-binary", "directory/with/tests", "// ");
//!     config.run_tests()
//! }
//! ```
//!
//! Now you can start adding tests to `directory/with/tests` and each test should
//! be automatically found and ran by goldentests whenever you run `cargo test`.
//! Here's a quick example of a test file that uses all of goldentest's features:
//!
//! ```python
//! import sys
//! 
//! print("hello!\nfriend!")
//! print("error!", file=sys.stderr)
//! sys.exit(3)
//! 
//! # Assuming 'python' is the command passed to TestConfig::new:
//! # args: -B
//! # expected exit status: 3
//! # expected stdout:
//! # hello!
//! # friend!
//! 
//! # expected stderr: error!
//! ```
//!
//! Check out the documentation in `TestConfig` for optional configuration.

pub mod config;
pub mod error;
mod diff_printer;

pub use config::TestConfig;
pub use error::TestError;
use diff_printer::DiffPrinter;

use colored::Colorize;
use difference::Changeset;
use shlex;

use std::fs::File;
use std::path::{ Path, PathBuf };
use std::io::Read;
use std::process::{ Command, Output };

pub type TestResult<T> = Result<T, error::TestError>;

struct Test {
    path: PathBuf,
    command_line_args: String,
    expected_stdout: String,
    expected_stderr: String,
    expected_exit_status: Option<i32>,
}

#[derive(PartialEq)]
enum TestParseState {
    Neutral,
    ReadingExpectedStdout,
    ReadingExpectedStderr,
}

fn find_tests(directory: &Path) -> TestResult<Vec<PathBuf>> {
    let mut tests = vec![];
    if directory.is_dir() {
        for entry in std::fs::read_dir(directory).map_err(TestError::IoError)? {
            let entry = entry.map_err(TestError::IoError)?;
            let path = entry.path();
            if path.is_dir() {
                tests.append(&mut find_tests(&path)?);
            } else {
                tests.push(path);
            }
        }
    }
    Ok(tests)
}

fn parse_test(test_path: &PathBuf, config: &TestConfig) -> TestResult<Test> {
    let path = test_path.clone();
    let mut command_line_args = String::new();
    let mut expected_stdout = String::new();
    let mut expected_stderr = String::new();
    let mut expected_exit_status = None;

    let mut file = File::open(test_path).map_err(TestError::IoError)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(TestError::IoError)?;

    let mut state = TestParseState::Neutral;
    for line in contents.lines() {
        if line.starts_with(&config.test_line_prefix) {
            // If we're currently reading stdout or stderr, append the line to the expected output
            if state == TestParseState::ReadingExpectedStdout {
                expected_stdout += line.strip_prefix(&config.test_line_prefix).unwrap();
                expected_stdout += "\n";
            } else if state == TestParseState::ReadingExpectedStderr {
                expected_stderr += line.strip_prefix(&config.test_line_prefix).unwrap();
                expected_stderr += "\n";

            // Otherwise, look to see if the line begins with a keyword and if so change state
            // (stdout/stderr) or parse an argument to the keyword (args/exit status).

            // args:
            } else if line.starts_with(&config.test_args_prefix) {
                command_line_args = line.strip_prefix(&config.test_args_prefix).unwrap().to_string();

            // expected stdout:
            } else if line.starts_with(&config.test_stdout_prefix) {
                state = TestParseState::ReadingExpectedStdout;
                // Append the remainder of the line to the expected stdout.
                // Both expected_stdout and expected_stderr are trimmed so extra spaces if this is
                // empty shouldn't matter.
                expected_stdout += &(line.strip_prefix(&config.test_stdout_prefix).unwrap().to_string() + "\n");

            // expected stderr:
            } else if line.starts_with(&config.test_stderr_prefix) {
                state = TestParseState::ReadingExpectedStderr;
                expected_stderr += &(line.strip_prefix(&config.test_stderr_prefix).unwrap().to_string() + "\n");

            // expected exit status:
            } else if line.starts_with(&config.test_exit_status_prefix) {
                let status = line.strip_prefix(&config.test_exit_status_prefix).unwrap().trim();
                expected_exit_status = Some(status.parse().map_err(TestError::ErrorParsingExitStatus)?);
            }
        } else {
            state = TestParseState::Neutral;
        }
    }

    Ok(Test { path, command_line_args, expected_stdout, expected_stderr, expected_exit_status })
}

/// Diff the given "stream" and expected contents of the stream.
/// Returns non-zero on error.
fn check_for_differences_in_stream(path: &Path, name: &str, stream: &[u8], expected: &str) -> i8 {
    let output_string = String::from_utf8_lossy(stream);
    let output = output_string.trim();
    let expected = expected.trim();

    let differences = Changeset::new(expected, output, "\n");
    let distance = differences.distance;
    if distance != 0 {
        println!("{}: Actual {} differs from expected {}:\n{}\n",
                path.display().to_string().bright_yellow(), name, name, DiffPrinter(differences));
        1
    } else {
        0
    }
}

fn check_for_differences(output: &Output, test: &Test) -> bool {
    let mut error_count = 0;
    if let Some(expected_status) = test.expected_exit_status {
        if let Some(actual_status) = output.status.code() {
            if expected_status != actual_status {
                error_count += 1;
                println!("{}: Expected an exit status of {} but process returned {}\n",
                       test.path.display().to_string().bright_yellow(), expected_status, actual_status);
            }
        } else {
            error_count += 1;
            println!("{}: Expected an exit status of {} but process was terminated by signal instead\n",
                    test.path.display().to_string().bright_yellow(), expected_status);
        }
    }

    error_count += check_for_differences_in_stream(&test.path, "stdout", &output.stdout, &test.expected_stdout);
    error_count += check_for_differences_in_stream(&test.path, "stderr", &output.stderr, &test.expected_stderr);
    error_count != 0
}

impl TestConfig {
    /// Recurse through all the files in self.path, parse them all,
    /// and run the target program with the arguments specified in the file.
    pub fn run_tests(&self) -> TestResult<()> {
        let files = find_tests(&self.test_path)?;
        let tests = files.iter()
            .map(|file| parse_test(file, self))
            .collect::<Vec<_>>();

        let mut failing_tests = 0;
        for test in tests {
            let test = test?;
            let mut args = vec![];

            // Avoid pushing an empty '' arg at the beginning
            let trimmed_args = test.command_line_args.trim();
            if !trimmed_args.is_empty() {
                args = shlex::split(trimmed_args).unwrap();
            }

            args.push(test.path.to_string_lossy().to_string());

            let output = Command::new(&self.binary_path).args(args).output().map_err(TestError::IoError)?;
            let new_error = check_for_differences(&output, &test);
            if new_error {
                failing_tests += 1;
            }
        }

        if failing_tests != 0 {
            println!("{} {} tests are failing\n", failing_tests.to_string().red(), "golden".bright_yellow());
            Err(TestError::ExpectedOutputDiffers)
        } else {
            Ok(())
        }
    }
}
