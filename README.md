
## golden-tests

Golden tests is a golden file testing library configured so that tests
can be created and edited from the test files alone without ever touching
the rust source code of the test.

### Getting Started

To get started plop this into your `Cargo.toml`:
```toml
golden-tests = "0.2.0"
```

And create an integration test in `tests/goldentests.rs`. The specific name
doesn't matter as long as the test can be picked up by cargo. A typical usage
looks like this:

```rust
use goldentests::{ TestConfig, run_tests };

#[test]
fn run_golden_tests() -> Result<(), Box<dyn Error>> {
    let config = TestConfig::new("target/debug/my-binary", "my-test-path", "// ");
    run_tests(&config)
}
```

This will tell golden-tests to find all files recursively in `my-test-path` and
run `target/debug/my-binary` to use the files in some way to produce the expected
output.  For example, if we're testing a compiler for a C-like language a test
file for us may look like this:

```c
puts("Hello, World!");

// args: --run
// expected stdout:
// Hello, World!
```

Note that there are test keywords `args:` and `expected stdout:` embedded in the comments.
This is what the `"// "` parameter was in the rust example. You can change this parameter
to change the prefix that golden-tests looks for when parsing a file. For most languages,
this should be a comment of some kind. E.g. if we were testing haskell, we would use `-- `
as the test-line prefix.

You can even configure the specific keywords used if you want. For any further information,
check out golden-test's documentation [here]().
