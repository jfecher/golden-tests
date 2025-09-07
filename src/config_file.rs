//! Find and load the `goldentests.toml` configuration file if it exists
use std::path::PathBuf;

use crate::TestConfig;

const CONFIG_FILE: &str = "goldentests.toml";

pub(crate) fn read_config_file(path: Option<PathBuf>) -> Option<TestConfig> {
    let path = path.or_else(find_config_file)?;
    let contents = std::fs::read(&path).ok()?;

    match toml::from_slice(&contents) {
        Ok(cfg) => Some(cfg),
        Err(error) => {
            eprintln!("Error while reading `{path:?}`: {error}");
            None
        }
    }
}

fn find_config_file() -> Option<PathBuf> {
    let mut path = PathBuf::from(CONFIG_FILE);
    // Search at most 5 parent directories
    let max_tries = 5;

    for _ in 0 .. max_tries {
        if path.try_exists().unwrap_or(false) {
            return Some(path);
        }
        path = PathBuf::from("..").join(&path);
    }
    None
}
