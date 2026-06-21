# 1.4.4

- Fixed an `--overwrite` bug where the expected stdout and stderr would sometimes not be separated by an empty line

# 1.4.3

- Added `--interactive`/`-i` flag to review failing test files one by one

# 1.4.2

- Added `glob` option to `goldentests.toml`
- Fixed a bug where `--overwrite` would add additional space to the start/end of output

# 1.4.1

- Added `-j<N>`/`--jobs <N>` argument to optionally specify the number of parallel jobs if the `parallel` feature is used
