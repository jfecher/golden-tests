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
mod runner;

pub use config::TestConfig;
pub use error::TestResult;
