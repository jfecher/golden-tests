use crate::config::TestConfig;
use crate::diff_printer::DiffPrinter;
use crate::error::{InnerTestError, TestResult};

use colored::Colorize;
use similar::TextDiff;

#[cfg(feature = "parallel")]
use rayon::iter::IntoParallelIterator;
#[cfg(feature = "parallel")]
use rayon::iter::ParallelIterator;

#[cfg(feature = "progress-bar")]
use indicatif::ProgressBar;

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

type InnerTestResult<T> = Result<T, InnerTestError>;

struct Test {
    path: PathBuf,
    command_line_args: String,
    command_line_args_after: String,
    expected_stdout: String,
    expected_stderr: String,
    expected_exit_status: Option<i32>,
    rest: String,
}

#[derive(PartialEq)]
enum TestParseState {
    Neutral,
    ReadingExpectedStdout,
    ReadingExpectedStderr,
}

fn find_tests(test_path: &Path) -> (Vec<PathBuf>, Vec<InnerTestError>) {
    let mut tests = vec![];
    let mut errors = vec![];

    if test_path.is_dir() {
        let read_dir = match std::fs::read_dir(test_path) {
            Ok(dir) => dir,
            Err(err) => return (tests, vec![InnerTestError::IoError(test_path.to_owned(), err)]),
        };

        for entry in read_dir {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(err) => {
                    errors.push(InnerTestError::IoError(test_path.to_owned(), err));
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
    } else {
        tests.push(test_path.into());
    }

    (tests, errors)
}

fn strip_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    s.strip_prefix(prefix).unwrap_or(s)
}

fn append_line(s: &mut String, line: &str) {
    *s += line;
    *s += "\n";
}

fn parse_test(test_path: &Path, config: &TestConfig) -> InnerTestResult<Test> {
    let mut command_line_args = String::new();
    let mut command_line_args_after = String::new();
    let mut expected_stdout = String::new();
    let mut expected_stderr = String::new();
    let mut expected_exit_status = None;
    let mut rest = String::new();

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

            // args after:
            } else if line.starts_with(&config.test_args_after_prefix) {
                command_line_args_after = strip_prefix(line, &config.test_args_after_prefix).to_string();

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
            } else {
                append_line(&mut rest, line);
            }
        } else {
            // Both expected_stdout and expected_stderr need a blank line at the end,
            // the order here implicitly skips that newline.
            if state == TestParseState::Neutral {
                append_line(&mut rest, line);
            }
            state = TestParseState::Neutral;
        }
    }

    // Remove \r from strings for windows compatibility. This means we
    // also can't test for any string containing "\r" unless this check
    // is improved to be more clever (e.g. only removing at the end of a line).
    let expected_stdout = expected_stdout.replace("\r", "");
    let expected_stderr = expected_stderr.replace("\r", "");

    Ok(Test {
        path: test_path.to_owned(),
        command_line_args,
        command_line_args_after,
        expected_stdout,
        expected_stderr,
        expected_exit_status,
        rest,
    })
}

fn write_expected_output_for_stream(
    file: &mut File,
    prefix: &str,
    marker: &str,
    expected: &[u8],
) -> std::io::Result<()> {
    // Doesn't handle \r correctly!
    // Strip leading and trailing newlines from the output
    let expected_stdout = String::from_utf8_lossy(expected).replace("\r", "");
    let lines: Vec<&str> = expected_stdout.trim().split('\n').collect();
    match lines.len() {
        // Don't write if there's nothing to write
        0 => Ok(()),
        1 if lines[0].len() == 0 => Ok(()),
        // If the line is short and nice, write that line
        1 if lines[0].len() < 80 => {
            write!(file, "{} ", marker)?;
            file.write_all(expected)?;
            writeln!(file, "")
        }
        // Otherwise we write it more longform
        _ => {
            writeln!(file, "{}", marker)?;
            for line in lines {
                file.write_all(prefix.as_bytes())?;
                file.write_all(line.as_bytes())?;
                writeln!(file, "")?;
            }
            writeln!(file, "")
        }
    }
}

fn overwrite_test(test_path: &PathBuf, config: &TestConfig, output: &Output, test: &Test) -> std::io::Result<()> {
    // Maybe copy the file so we don't remove it if we fail here?
    let mut file = File::create(test_path)?;

    file.write_all(test.rest.trim_end().as_bytes())?;
    writeln!(file, "")?;
    writeln!(file, "")?;

    if !test.command_line_args.is_empty() {
        writeln!(file, "{} {}", config.test_args_prefix, test.command_line_args.trim())?;
    }

    if !test.command_line_args_after.is_empty() {
        writeln!(
            file,
            "{} {}",
            config.test_args_after_prefix,
            test.command_line_args_after.trim()
        )?;
    }

    if Some(0) != output.status.code() {
        writeln!(
            file,
            "{} {}",
            config.test_exit_status_prefix,
            output.status.code().unwrap_or(0)
        )?;
    }

    write_expected_output_for_stream(
        &mut file,
        &config.test_line_prefix,
        &config.test_stdout_prefix,
        &output.stdout,
    )?;
    write_expected_output_for_stream(
        &mut file,
        &config.test_line_prefix,
        &config.test_stderr_prefix,
        &output.stderr,
    )
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
        #[cfg(feature = "progress-bar")]
        let progress = ProgressBar::new(test_sources.len() as u64);

        let results = into_iter(test_sources)
            .map(|file| {
                #[cfg(feature = "progress-bar")]
                progress.inc(1);
                let test = parse_test(&file, self)?;
                let mut args = vec![];

                // Avoid pushing an empty '' arg at the beginning
                let trimmed_args = test.command_line_args.trim();
                if !trimmed_args.is_empty() {
                    args = shlex::split(trimmed_args)
                        .ok_or_else(|| InnerTestError::ErrorParsingArgs(file.clone(), trimmed_args.to_owned()))?;
                }

                args.push(test.path.to_string_lossy().to_string());

                args.extend(shlex::split(&test.command_line_args_after).ok_or_else(|| {
                    InnerTestError::ErrorParsingArgs(file.clone(), test.command_line_args_after.to_owned())
                })?);

                let mut command = Command::new(&self.binary_path);
                command.args(args);
                let output =
                    command.output().map_err(|err| InnerTestError::CommandError(file.clone(), command, err))?;

                let differences = check_for_differences(&test.path, &output, &test);
                if self.overwrite_tests {
                    if let Err(InnerTestError::TestFailed { path, errors }) = differences {
                        overwrite_test(&file, self, &output, &test)
                            .map_err(|err| InnerTestError::IoError(file.to_owned(), err))?;

                        return Err(InnerTestError::TestUpdated { path, errors });
                    }
                }
                differences
            })
            .collect();

        #[cfg(feature = "progress-bar")]
        progress.finish_and_clear();
        results
    }

    /// Recurse through all the files in self.path, parse them all,
    /// and run the target program with the arguments specified in the file.
    pub fn run_tests(&self) -> TestResult<()> {
        let (tests, path_errors) = find_tests(&self.test_path);
        let outputs = self.test_all(tests);

        for error in path_errors {
            eprintln!("{}", error);
        }

        let total_tests = outputs.len();
        let mut failing_tests = 0;
        let mut can_be_fixed_with_overwrite_tests = 0;
        let mut updated_tests = 0;
        for result in &outputs {
            match result {
                Ok(_) => {}
                Err(InnerTestError::TestUpdated { .. }) => {
                    updated_tests += 1;
                }

                Err(InnerTestError::TestFailed { .. }) => {
                    can_be_fixed_with_overwrite_tests += 1;
                    failing_tests += 1;
                }

                Err(
                    InnerTestError::IoError(_, _)
                    | InnerTestError::CommandError(_, _, _)
                    | InnerTestError::ErrorParsingExitStatus(_, _, _)
                    | InnerTestError::ErrorParsingArgs(_, _),
                ) => {
                    failing_tests += 1;
                }
            }

            if let Err(err) = result {
                eprintln!("{}", err)
            }
        }

        if !self.overwrite_tests {
            println!(
                "ran {} {} tests with {} and {}\n",
                total_tests,
                "golden".bright_yellow(),
                format!("{} passing", total_tests - failing_tests).green(),
                format!("{} failing", failing_tests).red(),
            );
        } else {
            println!(
                "ran {} {} tests with {}, {} and {}\n",
                total_tests,
                "golden".bright_yellow(),
                format!("{} passing", total_tests - failing_tests).green(),
                format!("{} failing", failing_tests).red(),
                format!("{} updated", updated_tests).cyan(),
            );
        }

        if can_be_fixed_with_overwrite_tests > 0 {
            println!("Looks like you have failing tests. Review the output of each and fix any unexpected differences. When finished, you can use the --overwrite flag to automatically write the new output to the {} failing test file(s)", can_be_fixed_with_overwrite_tests);
        }

        if failing_tests != 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}
