
.PHONY: mod6_test
mod6_test:
	cargo test -p analysis

.PHONY: mod6_run_bin
mod6_run_bin:
	cargo run --bin cli -p analysis  -- analysis-project/example.log

.PHONY: mod6_clippy
mod6_clippy:
	cargo fmt -p analysis
	cargo clippy  --fix -p analysis


		