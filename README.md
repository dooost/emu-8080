# emu-8080
Functional Intel 8080 emulator written in Rust

## Tests
The project is tested with a number of 8080 test binaries that I could find online.

The `compare_i8080` tests run the test binaries side-by-side to [mohanson/i8080](https://github.com/mohanson/i8080) and compare the CPU state, 
and fail if the CPU states do not match. These by default do not show the last instruction that caused the inconsistency due to the performance hit
this creates with the current implementation, but it can be opted in using the `keep_old_state` flag, and the ignored 
`compare_on_8080exm_print_failed_instruction` already uses this.

The `diag_suites` test simply run the test binaries, so they can't really fail properly since there is no clear indicator besides the console output. 
Run them with `-- --nocapture` to see console outputs to see the binary's output, and if running multiple tests, use `--test-threads=1` to run 
only 1 at a time so console outputs don't conflict.

**Run tests in release since the larger test binaries take a very long time without optimizations!**

Run all tests:
```
cargo test --release
```

Run diag binaries:
```
cargo test --release  --test diag_suites -- --nocapture --test-threads=1
```

Run diag binaries:
```
cargo test --release  --test diag_suites -- --nocapture --test-threads=1
```

Run the largest test suite with failed instruction report (very slow):
```
cargo test --release -- compare_on_8080exm_print_failed_instruction --ignored
```
