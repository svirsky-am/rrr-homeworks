.PHONY: linting
linting:
	cargo fmt  --fix
	cargo clippy  --fix
	cargo check --fix

include docs/module1/run_build_test_mod1.mk
include docs/module2/run_build_test_mod2.mk
# module3
include blog-project/run_build_test_mod3.mk
# module 4
include image_ffi_project/run_build_test_mod4.mk

# module 5
include is-not-broken-app/scripts/module_5_fix_broken_app.mk