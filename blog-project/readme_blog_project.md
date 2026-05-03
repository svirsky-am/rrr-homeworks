
# Подготовка к сборке и запусу

# PostgreSQL
```sh
sudo apt update
sudo apt install postgresql-14 postgresql-client-14
sudo apt-get install protobuf-compiler
protoc --version # Для tonic 0.10+ рекомендуется protoc версии 3.15+.
```

Отключить хостовый postgresql-сервер
```sh
sudo systemctl stop postgresql
```
# Создать БД и задать пароль суперпользователю postgres
```sh
export DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base

export POSTRES_WORKDIR=.postgres_workdir/pgdata
	mkdir -p .logs
	pkill -9 postgres | true
	rm -rf ${POSTRES_WORKDIR}
	/usr/lib/postgresql/14/bin/initdb -D $POSTRES_WORKDIR
	cp -f blog-project/blog-server/scripts/* $POSTRES_WORKDIR/
	/usr/lib/postgresql/14/bin/pg_ctl -D $POSTRES_WORKDIR -l .logs/logfile_pg.log start
```
```sh
cd blog-project/blog-server
# 1. Применить миграции (если БД пуста)
cargo sqlx migrate run --database-url="$DATABASE_URL"

# 2. Создать кэш схемы для compile-time SQL checks
cargo sqlx prepare -- --database-url="$DATABASE_URL"

# 3. Собрать проект
cargo build -p blog-server

# 4. Запустить
cargo run -p blog-server

```
Запуск сервера с пересозданием базы:
```sh
make mod3_build_all
```

```sh 
export HOST=127.0.0.1
export PORT=8080
export JWT_SECRET=dev_super_secret_change_me_please
export  CORS_ORIGINS=http://localhost:8080
export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:8432/bank_api

```
Проверка 
Регистрация:
```sh
  curl -X POST http://localhost:8080/api/auth/register \
    -H "Content-Type: application/json" \
    -d '{"email": "user@example.com", "password": "secure123"}'
```
Логин:
```sh 
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}'
# или 
TOKEN_CLIENT=$(curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}' \
  | jq -r '.access_token')

echo $TOKEN_CLIENT
```
# Проверка gRPC через grpcurl

```sh
# Установите grpcurl (если нет)
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# Проверить список сервисов
grpcurl -plaintext localhost:50051 list

# Вызов метода (без аутентификации)
grpcurl -plaintext -d '{"limit": 5}' localhost:50051 blog.BlogService/ListPosts

# Вызов с токеном
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"title": "Hello", "content": "World"}' \
  localhost:50051 blog.BlogService/CreatePost
```


 Таблица конвертации ошибок: DomainError → gRPC Status

| DomainError |gRPC Status | HTTP-аналог
Validation(msg) | InvalidArgument |400 Bad Request
UserAlreadyExists | AlreadyExists | 409 Conflict
InvalidCredentials | Unauthenticated | 401 Unauthorized
Forbidden | PermissionDenied | 403 Forbidden
NotFound | NotFound | 404 Not Found
Database, Jwt, Internal  |  Internal | 500 Internal Server Error
