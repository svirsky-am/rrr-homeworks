



PHONY: mod5_1.2_check_build_broken_app
mod5_1.2_check_build_broken_app:
	cargo check \
		--manifest-path ./repos/origin-of-broken-app/Cargo.toml
	cargo test \
		--manifest-path ./repos/origin-of-broken-app/Cargo.toml

# cargo test \
# 		--manifest-path ./repos/origin-of-broken-app/Cargo.toml sums_even_numbers




PHONY: mod5_run_sanitize
mod5_run_sanitize:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly run --bin demo


PHONY: mod5_run_miri
mod5_run_miri:
# 	MIRIFLAGS=-Zmiri-env-forward=RUST_BACKTRACE cargo +nightly miri test 
	cargo  miri test 




PHONY: mod5_run_sanitize_sums_even_numbers
mod5_run_sanitize_sums_even_numbers:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers
# 	RUST_BACKTRACE=1 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers
	
PHONY: mod5_run_sanitize_sums_even_numbers
mod5_run_sanitize_sums_even_numbers:
	cargo +nightly build --lib
# 	cargo +nightly test --target x86_64-unknown-linux-gnu sums_even_numbers
	ASAN_OPTIONS=detect_leaks=0 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers
# 	RUST_BACKTRACE=1 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  sums_even_numbers




PHONY: mod5_run_sanitize_counts_non_zero_bytes
mod5_run_sanitize_counts_non_zero_bytes:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly build --lib -- -Zsanitizer=address
# 	cargo +nightly test --target x86_64-unknown-linux-gnu sums_even_numbers
	ASAN_OPTIONS=detect_leaks=0 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  counts_non_zero_bytes

PHONY: mod5_test_counts_non_zero_bytes_valgrind
mod5_test_counts_non_zero_bytes_valgrind:
	cargo valgrind test counts_non_zero_bytes
# 	ASAN_OPTIONS=detect_leaks=0 RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  --target x86_64-unknown-linux-gnu  counts_non_zero_bytes






PHONY: mod5_get_bench_of_fixed_broken_app
mod5_get_bench_of_fixed_broken_app:
	cargo bench  --bench baseline  > artifacts/generated/broken-app-benches.txt

PHONY: mod5_get_criterion_of_fixed_broken_app
mod5_get_criterion_of_fixed_broken_app:
	cargo bench  --bench criterion  > artifacts/generated/broken-app-criterion.txt


