use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::error::TestResult;

#[derive(Debug, Default)]
pub struct Globs {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl Globs {
    pub fn new(glob_strings: &[String]) -> TestResult<Globs> {
        let mut globs = Globs::default();

        let mut include_builder = GlobSetBuilder::new();
        let mut exclude_builder = GlobSetBuilder::new();

        println!("globs: {glob_strings:?}");
        for glob in glob_strings {
            println!("glob: {glob}");
            match glob.strip_prefix('!') {
                Some(glob) => {
                    exclude_builder.add(build_glob(glob)?);
                }
                None => {
                    include_builder.add(build_glob(glob)?);
                }
            }
        }

        let include = build_glob_set(include_builder)?;
        let exclude = build_glob_set(exclude_builder)?;

        if !include.is_empty() {
            globs.include = Some(include);
        }

        if !exclude.is_empty() {
            globs.exclude = Some(exclude);
        }
        Ok(globs)
    }

    pub fn is_match(&self, path: &Path) -> bool {
        let mut match_ = true;
        if let Some(include_set) = &self.include {
            match_ &= include_set.is_match(path);
        }
        if let Some(exclude_set) = &self.exclude {
            match_ &= !exclude_set.is_match(path);
        }
        match_
    }
}

fn build_glob(s: &str) -> TestResult<Glob> {
    Glob::new(s).map_err(|err| {
        eprintln!("Invalid glob: {}", err);
        ()
    })
}

fn build_glob_set(builder: GlobSetBuilder) -> TestResult<GlobSet> {
    builder.build().map_err(|err| {
        eprintln!("Unable to build glob set: {}", err);
        ()
    })
}
