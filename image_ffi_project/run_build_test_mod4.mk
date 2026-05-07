PHONY: mod4_image_ffi_project
mod4_image_ffi_project: 
	cargo build -p  "image_processor"


PHONY: mod4_build_blur_plugin
mod4_build_blur_plugin:
	cargo build -p  "blur_plugin"

PHONY: mod4_build_mirror_plugin
mod4_build_mirror_plugin:
	cargo build -p  "mirror_plugin"


PHONY: mod4_build_release
mod4_build_release:
	cargo build -p  "mirror_plugin" --release
	cargo build -p  "blur_plugin" --release
	cargo build -p  "image_processor" --release

PHONY: mod4_test
mod4_test:
	cargo test -p  "mirror_plugin" --release
	cargo test -p  "blur_plugin" --release
	RUST_LOG=debug cargo test -p  "image_processor" --test integration -- --nocapture

PHONY: mod4_test_lcov
mod4_test_lcov:
	cargo llvm-cov -p  "image_processor"
	cargo llvm-cov test --test integration_cov -p  "image_processor" -p  "blur_plugin" -p  "mirror_plugin" --html


PHONY: mod4_fmt
mod4_fmt:
	cargo fmt -p  "image_processor" -p  "blur_plugin" -p  "mirror_plugin"
	cargo clippy  --fix -p  "image_processor" -p  "blur_plugin" -p  "mirror_plugin"
	cargo check --fix -p  "image_processor" -p  "blur_plugin" -p  "mirror_plugin"