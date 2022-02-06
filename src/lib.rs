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
//!     let config = TestConfig::new("target/debug/my-binary", "directory/with/tests", "// ")?;
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

pub mod config;
mod diff_printer;
pub mod error;

pub use config::TestConfig;
use diff_printer::DiffPrinter;
use error::{InnerTestError, TestError};

use colored::Colorize;
#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;
#[cfg(feature = "parallel")]
use rayon::iter::ParallelIterator;
use shlex;
use similar::TextDiff;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub type TestResult<T> = Result<T, TestError>;

type InnerTestResult<T> = Result<T, InnerTestError>;

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

/// Expects that the given directory is an existing path
fn find_tests(directory: &Path) -> (Vec<PathBuf>, Vec<InnerTestError>) {
    let mut tests = vec![];
    let mut errors = vec![];

    let read_dir = match std::fs::read_dir(directory) {
        Ok(dir) => dir,
        Err(err) => return (tests, vec![InnerTestError::IoError(directory.to_owned(), err)]),
    };

    for entry in read_dir {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(err) => {
                errors.push(InnerTestError::IoError(directory.to_owned(), err));
                continue;
            }
        };

        if path.is_dir() {
            let (mut more_tests, mut more_errors) = find_tests(&path);
            tests.append(&mut more_tests);
            errors.append(&mut more_errors);
        } else {
            tests.push(path);
        }
    }

    (tests, errors)
}

fn strip_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    if s.starts_with(prefix) {
        &s[prefix.len()..]
    } else {
        s
    }
}

fn append_line(s: &mut String, line: &str) {
    *s += line;
    *s += "\n";
}

fn parse_test(test_path: &PathBuf, config: &TestConfig) -> InnerTestResult<Test> {
    let path = test_path.clone();
    let mut command_line_args = String::new();
    let mut expected_stdout = String::new();
    let mut expected_stderr = String::new();
    let mut expected_exit_status = None;

    let mut file = File::open(test_path).map_err(|err| InnerTestError::IoError(test_path.to_owned(), err))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| InnerTestError::IoError(test_path.to_owned(), err))?;

    let mut state = TestParseState::Neutral;
    for line in contents.lines() {
        if line.starts_with(&config.test_line_prefix) {
            // If we're currently reading stdout or stderr, append the line to the expected output
            if state == TestParseState::ReadingExpectedStdout {
                append_line(&mut expected_stdout, strip_prefix(line, &config.test_line_prefix))
            } else if state == TestParseState::ReadingExpectedStderr {
                append_line(&mut expected_stderr, strip_prefix(line, &config.test_line_prefix));

            // Otherwise, look to see if the line begins with a keyword and if so change state
            // (stdout/stderr) or parse an argument to the keyword (args/exit status).

            // args:
            } else if line.starts_with(&config.test_args_prefix) {
                command_line_args = strip_prefix(line, &config.test_args_prefix).to_string();

            // expected stdout:
            } else if line.starts_with(&config.test_stdout_prefix) {
                state = TestParseState::ReadingExpectedStdout;
                // Append the remainder of the line to the expected stdout.
                // Both expected_stdout and expected_stderr are trimmed so it
                // has no effect if the rest of this line is empty
                append_line(&mut expected_stdout, strip_prefix(line, &config.test_stdout_prefix));

            // expected stderr:
            } else if line.starts_with(&config.test_stderr_prefix) {
                state = TestParseState::ReadingExpectedStderr;
                append_line(&mut expected_stderr, strip_prefix(line, &config.test_stderr_prefix));

            // expected exit status:
            } else if line.starts_with(&config.test_exit_status_prefix) {
                let status = strip_prefix(line, &config.test_exit_status_prefix).trim();
                expected_exit_status = Some(status.parse().map_err(|err| {
                    InnerTestError::ErrorParsingExitStatus(test_path.to_owned(), status.to_owned(), err)
                })?);
            }
        } else {
            state = TestParseState::Neutral;
        }
    }

    // Remove \r from strings for windows compatibility. This means we
    // also can't test for any string containing "\r" unless this check
    // is improved to be more clever (e.g. only removing at the end of a line).
    let expected_stdout = expected_stdout.replace("\r", "");
    let expected_stderr = expected_stderr.replace("\r", "");

    Ok(Test {
        path,
        command_line_args,
        expected_stdout,
        expected_stderr,
        expected_exit_status,
    })
}

/// Diff the given "stream" and expected contents of the stream.
/// Returns non-zero on error.
fn check_for_differences_in_stream(name: &str, stream: &[u8], expected: &str, errors: &mut Vec<String>) {
    let output_string = String::from_utf8_lossy(stream).replace("\r", "");
    let output = output_string.trim();
    let expected = expected.trim();

    let differences = TextDiff::from_lines(expected, output);
    if differences.ratio() != 1.0 {
        errors.push(format!(
            "Actual {} differs from expected {}:\n{}",
            name,
            name,
            DiffPrinter(differences)
        ));
    }
}

fn check_exit_status(output: &Output, expected_status: Option<i32>, errors: &mut Vec<String>) {
    if let Some(expected_status) = expected_status {
        if let Some(actual_status) = output.status.code() {
            if expected_status != actual_status {
                errors.push(format!(
                    "Expected an exit status of {} but process returned {}\n",
                    expected_status, actual_status,
                ));
            }
        } else {
            errors.push(format!(
                "Expected an exit status of {} but process was terminated by signal instead\n",
                expected_status
            ));
        }
    }
}

fn check_for_differences(path: &Path, output: &Output, test: &Test) -> InnerTestResult<()> {
    let mut errors = vec![];
    check_exit_status(output, test.expected_exit_status, &mut errors);
    check_for_differences_in_stream("stdout", &output.stdout, &test.expected_stdout, &mut errors);
    check_for_differences_in_stream("stderr", &output.stderr, &test.expected_stderr, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        let path = path.to_owned();
        Err(InnerTestError::TestFailed { path, errors })
    }
}

// Returns a tuple of the number of (total_results, failing_results)
fn count_test_results(results: &[InnerTestResult<()>]) -> (usize, usize) {
    let mut failing = 0;
    for result in results {
        if let Err(_) = result {
            failing += 1;
        }
    }
    (results.len(), failing)
}

#[cfg(feature = "parallel")]
fn into_iter<T: IntoParallelIterator>(value: T) -> T::Iter {
    value.into_par_iter()
}

#[cfg(not(feature = "parallel"))]
fn into_iter<T: IntoIterator>(value: T) -> T::IntoIter {
    value.into_iter()
}

impl TestConfig {
    fn test_all(&self, test_sources: Vec<PathBuf>) -> Vec<InnerTestResult<()>> {
        into_iter(test_sources)
            .map(|file| {
                let test = parse_test(&file, self)?;
                let mut args = vec![];

                // Avoid pushing an empty '' arg at the beginning
                let trimmed_args = test.command_line_args.trim();
                if !trimmed_args.is_empty() {
                    args = shlex::split(trimmed_args)
                        .ok_or_else(|| InnerTestError::ErrorParsingArgs(file.clone(), trimmed_args.to_owned()))?;
                }

                args.push(test.path.to_string_lossy().to_string());

                let output = Command::new(&self.binary_path)
                    .args(args)
                    .output()
                    .map_err(|err| InnerTestError::IoError(file, err))?;

                check_for_differences(&test.path, &output, &test)?;
                Ok(())
            })
            .collect()
    }

    /// Recurse through all the files in self.path, parse them all,
    /// and run the target program with the arguments specified in the file.
    pub fn run_tests(&self) -> TestResult<()> {
        let (tests, path_errors) = find_tests(&self.test_path);
        let outputs = self.test_all(tests);

        for error in path_errors {
            eprintln!("{}", error);
        }

        for result in &outputs {
            if let Err(error) = result {
                eprintln!("{}", error);
            }
        }

        let (total_tests, failing_tests) = count_test_results(&outputs);

        println!(
            "ran {} {} tests with {} and {}\n",
            total_tests,
            "golden".bright_yellow(),
            format!("{} passing", total_tests - failing_tests).green(),
            format!("{} failing", failing_tests).red(),
        );

        if failing_tests != 0 {
            Err(TestError::TestErrors)
        } else {
            Ok(())
        }
    }
}
