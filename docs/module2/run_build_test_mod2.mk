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


.PHONY: cleanup-ports
cleanup-ports:
	@echo "Cleaning up zombie processes..."
	-pkill -f quote_server 2>/dev/null || true
	-pkill -f quote_client 2>/dev/null || true
	@sleep 1
	@echo "Done"

.PHONY: mod2-build-debug
mod2-build-debug: cleanup-ports
	cargo build -p streaming_quotes_project --bins
	
mod2-build-integerated-tests: mod2-build-debug
	cargo build -p streaming_quotes_project --bins
	RUST_LOG=info  cargo test -p streaming_quotes_project --test integration -- --nocapture



.PHONY: mod2-build-server-release
mod2-build-server-release:
	cargo build --release -p streaming_quotes_project --bins

# Проверка кода
# Форматирование
mod2-build-staff:
	cargo doc -p streaming_quotes_project --no-deps --open
	cargo clippy -p streaming_quotes_project --bins
	cargo fmt -p streaming_quotes_project