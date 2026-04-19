PHONY: mod3_run_cli
mod3_run_cli: 
	cargo run -p  "blog-client" balance --user test

PHONY: mod3_build_all
mod3_build_all: 
	cargo build -p  "blog-*"

PHONY: mod3_server_test
mod3_server_test: 
	cargo test -p  "blog-server"