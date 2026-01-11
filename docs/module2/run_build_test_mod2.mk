.PHONY: mod2-fmt-libs
mod2-fmt-libs:
	cargo fmt -p rr-parser-lib
	cargo clippy --allow-dirty --fix -p rr-parser-lib

.PHONY: mod2-build-streaming_quotes_project
mod2-build-streaming_quotes_project:
	cargo fix --lib -p streaming_quotes_project
	cargo fix --lib -p streaming_quotes_project --tests

.PHONY: mod2-lint-streaming_quotes_project
mod2-lint-streaming_quotes_project:
	cargo fix --lib -p streaming_quotes_project
	cargo fix --lib -p streaming_quotes_project --tests

.PHONY: mod2-build-integerated-tests
mod2-build-integerated-tests:
	cargo build -p streaming_quotes_project --bins
	cargo test -p streaming_quotes_project --test integration -- --nocapture
# cargo test -p streaming_quotes_project -- --nocapture


.PHONY: mod2-build-server
mod2-build-server:
	cargo build -p streaming_quotes_project --bins
# cargo test -p streaming_quotes_project -- --nocapture