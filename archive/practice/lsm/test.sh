# Pass `--test-threads=1` to prevent running tests in parallel.
# Tests use a data directory.
cargo test -- --test-threads=1 --show-output