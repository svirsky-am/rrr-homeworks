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
export DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:5432/blog_api
# export SECRET_ADMIN_PG_PASS=blog_pass
# JWT_SECRET=dev_super_secret_change_me_please
# CORS_ORIGINS=http://localhost:3000
EXCHANGE_API_URL=https://api.exchangerate-api.com/v4/latest


PHONY: mod3_server_run
mod3_server_run: 
	psql -d postgres -c "DROP DATABASE blog_api;" -h /tmp | true
	psql -d postgres -c "CREATE DATABASE blog_api; " -h /tmp
	psql -d postgres -c "CREATE USER blog_admin WITH PASSWORD 'blog_pass'" -h /tmp | true;
	cargo build -p  "blog-server"
	cargo run -p  "blog-server"

# 	psql -U $$USER -h localhost -d postgres "GRANT ALL PRIVILEGES ON DATABASE bank_api TO blog_admin;"

