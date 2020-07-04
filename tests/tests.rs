use goldentests::TestConfig;
use std::error::Error;

#[test]
fn run_goldentests() -> Result<(), Box<dyn Error>> {
    let config = TestConfig::new("python", "examples", "# ");
    config.run_tests()
}
