# 📝 Blog Project

Полноценное рабочее пространство (workspace) с бэкендом, клиентской библиотекой, CLI и WASM-фронтендом.

## 🏗️ Архитектура

├── blog-server/ # Actix-web + SQLx + gRPC сервер
├── blog-client/ # Библиотека клиента (HTTP + gRPC)
├── blog-cli/ # CLI-интерфейс на базе blog-client
└── blog-wasm/ # WASM-фронтенд (только HTTP)



# Подготовка к сборке и запусу

# PostgreSQL
```sh
sudo apt update
sudo apt install postgresql-14 postgresql-client-14
sudo apt-get install protobuf-compiler
protoc --version # Для tonic 0.10+ рекомендуется protoc версии 3.15+.
cargo install wasm-pack --locked

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




Тестирование через CLI

```sh
# Регистрация
cargo run -p blog-cli -- register -u testuser -e test@example.com -p pass123
# Логин
cargo run -p blog-cli -- login -u testuser -p pass123
# Создание поста
cargo run -p blog-cli -- create -t "Hello" -c "World"
# Список постов
cargo run -p blog-cli -- list
# Через gRPC (добавьте флаг --grpc)
cargo run -p blog-cli -- --grpc list
```

Запуск WASM-фронтенда

```sh
# Сборка WASM-модуля
cd blog-wasm
wasm-pack build --target web

# Запуск локального сервера
cd ..
python3 -m http.server 8000 --directory blog-wasm

# Откройте в браузере:
# http://localhost:8000
```


## Синхронизация proto-схем
При изменении blog.proto:
```sh 
# 1. Обновите файл в blog-server/proto/
# 2. Скопируйте в blog-client/proto/
cp blog-server/proto/blog.proto blog-client/proto/
# 3. Пересоберите оба крейта
cargo build -p blog-server
cargo build -p blog-client
```