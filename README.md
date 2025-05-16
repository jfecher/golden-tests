
## Golden Tests

[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fjfecher%2Fgolden-tests%2Fbadge&style=flat)](https://actions-badge.atrox.dev/jfecher/golden-tests/goto)
[![crates.io](https://img.shields.io/crates/v/goldentests)](https://crates.io/crates/goldentests)
[![docs.rs](https://docs.rs/goldentests/badge.svg)](https://docs.rs/goldentests)

Golden tests is a golden file testing library configured so that tests
can be created and edited from the test files alone without ever touching
the source code of your compiler, interpreter, or other tool.

### Why golden tests?

Golden tests allow you to specify the output of
some command within a file and automatically ensure
that that output doesn't change. If it does, goldentests
will show an error-diff showing the expected and actual
output. This way, whenever the output of something changes
a human can see the change and decide if it should be kept
or is a bug and should be reverted.

### What are golden tests useful for?

Golden tests are especially useful for applications that
take a file as input and produce output of some kind. For
example: compilers and config-parsers (well, parsers in general)
are two such applications that can benefit from automated golden
tests. In the case of a config parser, you would be able to
provide many config examples as tests and ensure that your
parser was able to read the files with the expected stdout/stderr
output and exit code.

### Example Output

![example image](example.png)

### Getting Started

As of version 1.1, there are now two ways to use goldentests - either as a
standalone binary or as a rust integration test. If you want to run it as
a binary, continue on. If not, skip ahead to the next section. With that
out of the way, we can install goldentests via:

```sh
$ cargo install goldentests --features binary
```

An example usage looks like this:

```sh
$ goldentests /bin/python path-to-tests '# '
```

This will tell goldentests to run `/bin/python` on each file in the `path-to-tests`
directory. You'll likely want to alias this command with your preferred arguments
for easier testing. An example test for us may look like this:

```py
print("Hello, World!")

# args: -b
# expected stdout:
# Hello, World!
```

This file tells goldentests to run the command `/bin/python -b path-to-tests/example.py` and issue
an error if the output of the command is not "Hello, World!".

Note that there are test keywords `args:` and `expected stdout:` embedded in the comments.
This is what the `'# '` parameter was when we invoked goldentests. You can change this parameter
to change the prefix that goldentests looks for when parsing a file. For most languages,
this should be a comment of some kind. E.g. if we we're testing haskell, we would use `-- `
as the test-line prefix.

#### As a rust integration test

The second way to use goldentests is as a rust library for writing
integration tests. Using this method will have `goldentests` run
each time you call `cargo test`. To get started plop this into your `Cargo.toml`:
```toml
goldentests = "1.3"
```

And create an integration test in `tests/goldentests.rs`. The specific name
doesn't matter as long as the test can be picked up by cargo. A typical usage
looks like this:

```rust
use goldentests::{ TestConfig, TestResult };

#[test]
fn run_golden_tests() -> TestResult<()> {
    let config = TestConfig::new("target/debug/my-binary", "my-test-path", "// ");
    config.run_tests()
}
```

This will tell goldentests to find all files recursively in `my-test-path` and
run `target/debug/my-binary` to use the files in some way to produce the expected
output.  For example, if we're testing a compiler for a C-like language a test
file for us may look like this:

```c
puts("Hello, World!");

// args: --run
// expected stdout:
// Hello, World!
```

This will run the command `target/debug/my-binary --run my-test-path/example.c` and will issue
an error if the output of the command is not "Hello, World!".

Note that there are test keywords `args:` and `expected stdout:` embedded in the comments.
This is what the `"// "` parameter was in the rust example. You can change this parameter
to change the prefix that goldentests looks for when parsing a file. For most languages,
this should be a comment of some kind. E.g. if we we're testing haskell, we would use `-- `
as the test-line prefix.

It can sometimes be convenient when using golden-tests via the Rust testing setup to have
arguments that are included by default for every program. These can be added by setting
the `base_args` and `base_args_after` fields of the `TestConfig` object. Among other things,
this can be used to easily re-run a set of tests with different arguments.

### Advanced Usage

Here is the full set of keywords goldentests looks for in the file:

- `args: <single-line-string>`: Anything after this keyword will be used as command-line arguments for the
  program that was specified when creating the `TestConfig`. These arguments will all be placed before the file argument.
- `args after: <single-line-string>`: Anything after this keyword will be used as command-line arguments for the
  program that was specified when creating the `TestConfig`. These arguments will all be placed after the file argument.
- `expected stdout: <multi-line-string>`: This keyword will continue reading characters, appending
  them to the expected stdout output until it reaches a line that does not start with the test prefix
  ("// " in the example above). If the stdout when running the program differs from the string given here,
  an appropriate error will be issued with a given diff. Defaults to `""`.
- `expected stderr: <multi-line-string>`: The same as `expected stdout:` but for the `stderr` stream. Also
  defaults to `""`.
- `expected exit status: [i32]`: If specified, goldentests will issue an error if the exit status differs
  to what is expected. Defaults to `None` (exit status is ignored by default).


You can even configure the specific keywords used if you want. For any further information,
check out goldentest's documentation [here](https://docs.rs/goldentests).

### Automatically updating tests

Optionally, tests can be automatically updated by passing the `--overwrite`
flag when running goldentests as a standalone program, or by setting the
`overwrite_tests` flag when running as a rust library. Doing this will update
the expected output in each file so that it matches the actual output. Since
this is all automatic, make sure to manually review any changes before using
this flag.

### Features

Given below is a list of each crate feature as well as whether it is enabled by default:

- `binary` (disabled): Build `goldentests` as a standalone binary rather than a rust testing library
- `progress-bar` (disabled): Display a progress bar while testing. Useful if running many tests but `cargo test` hides the output of tests until it finishes by default so this is by default only enabled if `binary` is enabled. If you want to use this with `cargo test`, you can still enable this and make sure to pass the `no-capture` flag to `cargo test` when running.
- `parallel` (enabled): Run tests in parallel.
