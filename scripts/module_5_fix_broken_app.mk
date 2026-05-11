
PHONY: mod5_run_sanitize
mod5_run_sanitize:
	RUSTFLAGS="-Zsanitizer=address" cargo +nightly run --bin demo



PHONY: mod5_get_bench_of_fixed_broken_app
mod5_get_bench_of_fixed_broken_app:
	cargo bench  --bench baseline  > artifacts/generated/broken-app-benches.txt

PHONY: mod5_get_criterion_of_fixed_broken_app
mod5_get_criterion_of_fixed_broken_app:
	cargo bench  --bench criterion  > artifacts/generated/broken-app-criterion.txt