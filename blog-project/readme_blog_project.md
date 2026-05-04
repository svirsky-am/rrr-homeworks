# 📝 Blog Project

Полноценное рабочее пространство (workspace) с бэкендом, клиентской библиотекой, CLI и WASM-фронтендом.

## 🏗️ Архитектура

├── blog-server/ # Actix-web + SQLx + gRPC сервер
├── blog-client/ # Библиотека клиента (HTTP + gRPC)
├── blog-cli/ # CLI-интерфейс на базе blog-client
└── blog-wasm/ # WASM-фронтенд (только HTTP)



# Подготовка к сборке и запусу
Целевая система: `ubuntu 24 LTS`.
Перечень необходимых пакетов:
```sh
sudo apt update
sudo apt install postgresql-14 postgresql-client-14
sudo apt-get install protobuf-compiler
protoc --version # Для tonic 0.10+ рекомендуется protoc версии 3.15+.
cargo install wasm-pack --locked
```
В проекте не используется хостовая служба systemd  postgresql-сервера, поэтому ее можно отключить:
```sh
sudo systemctl stop postgresql
```

# Подготовка БД
Дельнейшие шаги выполняет make-таргет `mod3_reinit_db`, но если делать самостоятельно , то:

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
# Запуск сервера blog-server
Дельнейшие шаги выполняет make-таргет `mod3_server_run`, но если делать самостоятельно , то:
```sh
export HOST=127.0.0.1
export PORT=8080
export JWT_SECRET=dev_super_secret_change_me_please
export  CORS_ORIGINS=http://localhost:8080
export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:8432/bank_api
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
или запуск сервера с пересозданием базы:
```sh
make mod3_server_run
```

# Пользовательский сценарий после запуска сервера с помощью curl 
Регистрация:
```sh
# Регистрация
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"secure123"}'
# Логин
export TOKEN_CLIENT=$(curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"secure123"}' \
  | jq -r '.token')
echo "Токен получен (первые 20 символов): ${TOKEN_CLIENT:0:20}..."
# Создание поста (с токеном)
curl -X POST http://localhost:3000/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN_CLIENT}" \
  -d '{"title":"Hello","content":"World"}'

curl -X GET http://localhost:3000/api/posts/debug/routes \
  -H "Authorization: Bearer ${TOKEN_CLIENT}"
# Список постов
curl http://localhost:3000/api/posts?limit=5&offset=0
```

## Тесты blog-server
### Через make- таргеты

Простой и поддерживаемый вариант:
```sh
make mod3_integration_test_debug
make mod3_grpc_integration_test_debug 
```

### Через sh
```sh
export JWT_SECRET=dev_super_secret_change_me_please
export TEST_DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base_test
cargo check -p blog-server --bin blog-server
	psql -d postgres -p 8432 -c "DROP DATABASE r_blog_base_test;" -h /tmp | true
	psql -d postgres -p 8432 -c "CREATE DATABASE r_blog_base_test; " -h /tmp
	TEST_DATABASE_URL=$(TEST_DATABASE_URL) RUST_LOG=actix_web=debug cargo test -p blog-server --test integration -- --nocapture --test-threads=1
cargo build -p blog-server
	TEST_DATABASE_URL=$(TEST_DATABASE_URL) RUST_LOG=tonic=debug  cargo test -p blog-server --test grpc_integration -- --nocapture --test-threads=1
```

## Таблица конвертации ошибок в blog-server: DomainError -> gRPC Status
| DomainError |gRPC Status | HTTP-аналог |
|-|-|-|
|Validation(msg) | InvalidArgument |400 Bad Request |
|UserAlreadyExists | AlreadyExists | 409 Conflict|
|InvalidCredentials | Unauthenticated | 401 Unauthorized |
|Forbidden | PermissionDenied | 403 Forbidden|
|NotFound | NotFound | 404 Not Found |
|Database, Jwt, Internal  |  Internal | 500 Internal Server Error |

# blog-cli
```sh
# Регистрация
cargo run -p blog-cli -- register -u testuser46456 -e test456345645@example.com -p pass123
# Логин
cargo run -p blog-cli -- login -u testuser46456 -p pass123
# Создание поста
cargo run -p blog-cli -- create -t "Hello by cli" -c "World"
# Список постов
cargo run -p blog-cli -- list
# Через gRPC (добавьте флаг --grpc)
cargo run -p blog-cli -- --grpc list
```
## Проверка gRPC через grpcurl

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


# Запуск WASM-фронтенда
Быстрый запуск:
```sh
make mod3_blog_wasm_build
```
Или пошагово:
```sh
# Сборка WASM-модуля
cd blog-wasm
wasm-pack build --target web
# Запуск локального сервера
cd ..
python3 -m http.server 8000 --directory blog-wasm
```
Откройте в браузере:
http://localhost:8000

# Синхронизация proto-схем
При изменении blog.proto:
```sh 
# 1. Обновите файл в blog-server/proto/
# 2. Скопируйте в blog-client/proto/
cp blog-server/proto/blog.proto blog-client/proto/
# 3. Пересоберите оба крейта
cargo build -p blog-server
cargo build -p blog-client
```