use goldentests::{TestConfig, TestResult};

#[test]
fn run_goldentests_example() -> TestResult<()> {
    let config = TestConfig::new("python", "examples", "# ")?;
    config.run_tests()
}

#[test]
fn test_overwrite() -> TestResult<()> {
    // Test this without the filesystem? Is it worth it?

    let overwrite_tests_dir = "overwrite_tests";
    let overwrite_test = format!("{}/_gen.py", overwrite_tests_dir);
    let test = "import sys; print('stdout'); print('stderr\\nwith\\nnewline\\n', file=sys.stderr);";

    // Nuke the entire dir to makesure everything is the same for each test
    std::fs::remove_dir_all(overwrite_tests_dir).unwrap();
    std::fs::create_dir(overwrite_tests_dir).unwrap();
    std::fs::write(&overwrite_test, test).unwrap();

    let mut gen = TestConfig::new("python", overwrite_tests_dir, "# ")?;
    gen.overwrite_tests = true;
    assert!(
        gen.run_tests().is_ok(),
        "Expected the tests to pass but also overwrite"
    );

    // Now the tests should work, and the file should differ
    let fixed_test = std::fs::read_to_string(&overwrite_test).unwrap();
    assert_ne!(fixed_test, test, "The test file should have been updated");
    
    // Tests should now pass
    let check = TestConfig::new("python", overwrite_tests_dir, "# ")?;
    check.run_tests()
}
