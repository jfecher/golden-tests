use goldentests::{ TestConfig, TestResult };

#[test]
fn run_goldentests() -> TestResult<()> {
    let config = TestConfig::new("python", "examples", "# ");
    config.run_tests()
}
