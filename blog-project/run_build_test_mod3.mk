PHONY: mod3_run_cli
mod3_run_cli: 
	cargo run -p  "blog-client" balance --user test

PHONY: mod3_build_all
mod3_build_all: 
	cargo build -p  "blog-*"

PHONY: mod3_server_test
mod3_server_test: 
	cargo test -p  "blog-server"

export HOST=127.0.0.1
export PORT=8080
export JWT_SECRET=dev_super_secret_change_me_please
export  CORS_ORIGINS=http://localhost:8080
export DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base
export TEST_DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base_test
export PG_SOCKET_DIR=/tmp/blog_server_work_dir



POSTRES_WORKDIR=.postgres_workdir/pgdata
PHONY: mod3_reinit_postgres_server
mod3_reinit_postgres_server:
	mkdir -p .logs
	pkill -9 postgres | true
	rm -rf $(POSTRES_WORKDIR)
	/usr/lib/postgresql/14/bin/initdb -D $(POSTRES_WORKDIR)
	cp -f blog-project/blog-server/scripts/* $(POSTRES_WORKDIR)/
	/usr/lib/postgresql/14/bin/pg_ctl -D $(POSTRES_WORKDIR) -l .logs/logfile_pg.log start
# 	psql -d postgres -c "CREATE USER blog_admin WITH PASSWORD 'blog_pass'" -h /tmp | true;



PHONY: mod3_reinit_db
mod3_reinit_db:
	psql -d postgres -p 8432 -c "DROP DATABASE r_blog_base;" -h /tmp | true
	psql -d postgres -p 8432 -c "CREATE DATABASE r_blog_base; " -h /tmp
	psql -d postgres -p 8432 -c "CREATE USER blog_admin WITH PASSWORD 'blog_pass'" -h /tmp | true;
	psql -d postgres -p 8432 -c "GRANT ALL PRIVILEGES ON DATABASE r_blog_base TO blog_admin;" -h /tmp | true;
	psql -d postgres -p 8432 -c "DROP DATABASE r_blog_base;" -h /tmp | true
	psql -d postgres -p 8432 -c "CREATE DATABASE r_blog_base; " -h /tmp
	DATABASE_URL=$(DATABASE_URL) cargo sqlx migrate run --source blog-project/blog-server/migrations
# 	rm -rf .sqlx
# 	cargo sqlx  prepare --workspace --database-url=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base

PHONY: mod3_apply_migration
mod3_apply_migration:

	cargo sqlx  prepare --workspace blog-server --database-url=$(DATABASE_URL)

# 	psql -U $$USER -h localhost -d postgres "GRANT ALL PRIVILEGES ON DATABASE r_blog_base TO blog_admin;"
PHONY: mod3_server_run
mod3_server_run: mod3_reinit_db
	RUST_LOG=debug  cargo build -p  "blog-server"
	RUST_LOG=debug  cargo run -p  "blog-server"


PHONY: mod3_integration_test_diagnose_faile
mod3_integration_test_diagnose_faile:
	cargo check -p blog-server --bin blog-server
	psql -d postgres -p 8432 -c "DROP DATABASE r_blog_base_test;" -h /tmp | true
	psql -d postgres -p 8432 -c "CREATE DATABASE r_blog_base_test; " -h /tmp
	TEST_DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base_test \
		RUST_LOG=actix_web=debug,actix_server=debug,reqwest=debug \
		cargo test -p blog-server --test integration -- --nocapture --test-threads=1 2>&1 | tee .logs/test_debug.log
# grep -A5 "thread 'test_" .logs/test_debug.log
# grep "actix_web::middleware::logger.*401\|404\|500\|400" .logs/test_debug.log
# grep "Adding service" .logs/test_debug.log


PHONY: mod3_integration_test_run_single_test
mod3_integration_test_run_single_test:
	RUST_LOG=debug cargo test -p blog-server --test integration test_unauthorized_access -- --nocapture
#	RUST_LOG=debug cargo test -p blog-server --test integration test_posts_pagination -- --nocapture
# 	RUST_LOG=debug cargo test -p blog-server --test integration test_unauthorized_access -- --nocapture


PHONY: mod3_integration_test_debug
mod3_integration_test_debug:
	cargo check -p blog-server --bin blog-server
	psql -d postgres -p 8432 -c "DROP DATABASE r_blog_base_test;" -h /tmp | true
	psql -d postgres -p 8432 -c "CREATE DATABASE r_blog_base_test; " -h /tmp
	TEST_DATABASE_URL=$(TEST_DATABASE_URL) RUST_LOG=actix_web=debug cargo test -p blog-server --test integration -- --nocapture --test-threads=1
# 	RUST_LOG=actix_web=debug cargo test -p blog-server --test integration test_full_flow_single_client -- --nocapture 2>&1 | grep -E "Adding service|POST /api/posts|404"
# 	cargo check -p blog-server --test integration --message-format=short 2>&1 | head -30
# 	RUST_LOG=debug cargo test -p blog-server --test integration -- --nocapture --test-threads=2

PHONY: mod3_grpc_integration_test_debug
mod3_grpc_integration_test_debug:
	cargo build -p blog-server
	TEST_DATABASE_URL=$(TEST_DATABASE_URL) RUST_LOG=tonic=debug  cargo test -p blog-server --test grpc_integration -- --nocapture --test-threads=1


PHONY: mod3_blog_client_build
mod3_blog_client_build:
	cargo clean -p blog-client
	cargo check -p blog-client
	RUST_LOG=debug  cargo build -p  "blog-client"
# 	RUST_LOG=debug  cargo build -p  "blog-client"	 --no-default-features --features grpc

PHONY: mod3_blog_cli_build
mod3_blog_cli_build: mod3_blog_client_build
	RUST_LOG=debug  cargo build -p  "blog-cli"	

PHONY: mod3_blog_wasm_build
mod3_blog_wasm_build:
	bash -C blog-project/blog-wasm/pack_and_run_ui.sh


PHONY: mod3_blog_build_all_debug
mod3_blog_build_all_debug:
	cargo build -p blog-server
	cargo build -p blog-client
	cargo build -p blog-cli
	bash -C blog-project/blog-wasm/pack_and_run_ui.sh

PHONY: mod3_blog_build_all_release
mod3_blog_build_all_release:
	cargo build -p blog-server --release
	cargo build -p blog-client --release
	cargo build -p blog-cli --release
	bash -C blog-project/blog-wasm/pack_ui.sh


PHONY: mod3_run_release
mod3_run_release:
	target/release/blog-server