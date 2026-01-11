.PHONY: linting
linting:
	cargo fmt
	cargo clippy
	cargo check

include docs/module1/run_build_test_mod1.mk
include docs/module2/run_build_test_mod2.mk