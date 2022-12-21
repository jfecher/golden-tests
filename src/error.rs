use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use colored::Colorize;

pub type TestResult<T> = Result<T, TestError>;

#[derive(Debug)]
pub enum TestError {
    MissingTests(PathBuf),
    ExpectedDirectory(PathBuf),
    TestErrors,
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TestError::*;
        match self {
            TestErrors => f.write_str("The expected test output differs"),
            MissingTests(path) => write!(f, "Failed to locate test files {}", path.display()),
            ExpectedDirectory(path) => {
                let msg = "The path given for test files should be a directory ";
                write!(f, "{}{}", msg, path.display())
            }
        }
    }
}

impl Error for TestError {}

// Inner test errors shouldn't be visible to the end-user,
// they'll all be reported internally after running the tests
pub(crate) enum InnerTestError {
    TestUpdated { path: PathBuf, errors: Vec<String> },
    TestFailed { path: PathBuf, errors: Vec<String> },
    IoError(PathBuf, std::io::Error),
    CommandError(PathBuf, std::process::Command, std::io::Error),
    ErrorParsingExitStatus(PathBuf, /*status*/ String, std::num::ParseIntError),
    ErrorParsingArgs(PathBuf, /*args*/ String),
}

impl fmt::Display for InnerTestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = |path: &PathBuf| path.to_string_lossy().bright_yellow();

        match self {
            InnerTestError::TestFailed { path, errors } => {
                for (i, error) in errors.iter().enumerate() {
                    write!(f, "{}: {}", s(path), error)?;
                    if i + 1 != errors.len() {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
            InnerTestError::TestUpdated { path, errors } => {
                for (i, error) in errors.iter().enumerate() {
                    write!(f, "{}: Updated {}", s(path), error)?;
                    if i + 1 != errors.len() {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
            InnerTestError::IoError(path, error) => {
                writeln!(f, "{}: {}", s(path), error)
            }
            InnerTestError::CommandError(path, command, error) => {
                writeln!(f, "{}: Error running `{:?}`: {}", s(path), command, error)
            }
            InnerTestError::ErrorParsingExitStatus(path, status, error) => {
                writeln!(f, "{}: Error parsing exit status '{}': {}", s(path), status, error)
            }
            InnerTestError::ErrorParsingArgs(path, args) => {
                writeln!(f, "{}: Error parsing test args: {}", s(path), args)
            }
        }
    }
}
