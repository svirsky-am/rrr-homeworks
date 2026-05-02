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
# 	cargo sqlx  prepare --workspace --database-url=$(DATABASE_URL)

PHONY: mod3_apply_migration
mod3_apply_migration:

	cargo sqlx  prepare --workspace blog-server --database-url=$(DATABASE_URL)

# 	psql -d postgres -c "CREATE USER blog_admin WITH PASSWORD 'blog_pass'" -h /tmp | true;
# 	psql -U $$USER -h localhost -d postgres "GRANT ALL PRIVILEGES ON DATABASE r_blog_base TO blog_admin;"
PHONY: mod3_server_run
mod3_server_run:
	RUST_LOG=debug  cargo build -p  "blog-server"
	RUST_LOG=debug  cargo run -p  "blog-server"

# 	psql -U $$USER -h localhost -d postgres "GRANT ALL PRIVILEGES ON DATABASE bank_api TO blog_admin;"

