use std::fmt;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug)]
pub enum TestError {
    ExpectedOutputDiffers,
    IoError(std::io::Error),
    ErrorParsingExitStatus(std::num::ParseIntError),
    MissingTests(PathBuf),
    ExpectedDirectory(PathBuf),
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::ExpectedOutputDiffers => f.write_str("The expected test output differs"),
            Self::IoError(err) => fmt::Display::fmt(err, f),
            Self::ErrorParsingExitStatus(err) => write!(f, "Error parsing exit status: {}", err),
            Self::MissingTests(path) => write!(f, "Failed to locate test files {}", path.display()),
            Self::ExpectedDirectory(path) => write!(f, "The path given for test files should be a directory {}", path.display()),
        }
    }
}

impl Error for TestError { } 
