
.PHONY: mod6_test
mod6_test:
	cargo test -p analysis

.PHONY: mod6_run_bin
mod6_run_bin:
	cargo run --bin cli -p analysis  -- analysis-project/example.log 