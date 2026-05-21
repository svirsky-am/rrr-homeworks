.PHONY: linting
linting:
	cargo fmt  --fix
	cargo clippy  --fix
	cargo check --fix

# module 6
include analysis-project/module6_analysis.mk