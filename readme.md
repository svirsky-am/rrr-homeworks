# 5. Модуль 5 ("поломанное" приложение с утечками `broken_app`)
Подробности в [расшериенном описании решения](is-not-broken-app/is-not-broken-app-readme.md) 

# 4. Модуль 4 (Безопасный обработчик изображений)
## 4.1 Релизная сборка плагинов и утилиты `image_processor`
```sh
cargo build -p  "mirror_plugin" --release
cargo build -p  "blur_plugin" --release
cargo build -p  "image_processor" --release
```
## 4.2. Запуск с `Mirror Plugin`:
```sh 
target/release/image_processor \
  --input image_ffi_project/test/input.png \
  --output output/release_version_mirror.png \
  --plugin image_ffi_project/mirror_plugin \
  --params image_ffi_project/test/params_mirror.json \
  --plugin-path target/release

```
## 4.3. Run with `Blur Plugin`:
```sh 
target/release/image_processor \
  --input image_ffi_project/test/input.png \
  --output output/release_version_blur.png \
  --plugin image_ffi_project/blur_plugin \
  --params image_ffi_project/test/params_blur.json \
  --plugin-path target/release
```
## 4.4 extra
Подробности в [расшериенном описании решения](image_ffi_project/image_ffi_project.md) 

# 3. Модуль 3 (клиент-серверное blog)

## 3.1 Описание
- `blog-server` - сервис ведения блога
- `blog-cli` - консольная утилита управления постами 
- `blog-wasm` - веб-клиент управления постами

Используемые крейты:
├── blog-server/ # Actix-web + SQLx + gRPC сервер
├── blog-client/ # Библиотека клиента (HTTP + gRPC)
├── blog-cli/ # CLI-интерфейс на базе blog-client
└── blog-wasm/ # WASM-фронтенд (только HTTP)
Требования к эксплуатации:
  - сервер blog-server: Ubuntu 24;
  - полнеченный доступ к https://crates.io/;
  - rustup > 1.28.2;
  - cargo > 1.90.0;
  - клиент blog-cli: Ubuntu 24/debian12;
  - web с client: хост с браузером и сетевой доступоностью к blog-server;
  - базовые навыки администрирования postgeSQL;
  - понимения основ работы с компьютерным сетями TCP/IP;

## 3.2 Быстрый запуск
```sh
make mod3_reinit_postgres_server # чтобы не дергать сервис postgre каждый раз
make mod3_reinit_db # чтобы не перетирать данные каждый раз при запуске
make mod3_server_tests_release # проверка работоспособности сервера перед запуском в продакшене
make mod3_blog_build_and_run_all_release
```

## 3.3 Описание ручной сборки и запуска сервера `blog-server` 
```sh
# Сборка релизного 'blog-server' без подключения к БД
SQLX_OFFLINE=true cargo build -p blog-server --release
# инициализируем базу данных и запускаем postgreSQL в пользовательском пространстве
sh ./blog-project/blog-server/scripts/prepare_db.sh
# Непосредственный запуск сервера
export  DATABASE_URL=postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base
target/release/blog-server
```
## 3.4 ## Консольная утилита управления постами `blog-cli`
Релизная утилита `blog-cli` будет лежать по пути `target/release/blog-cli`. Подробности использования:
```sh
target/release/blog-cli --help
```
## 3.5 ## Графическая оболочка `blog-wasm`
```sh
./blog-project/blog-wasm/pack_and_run_ui.sh
```
Далее нужно открыть в браузере `http://0.0.0.0:8000/`.
## Подробности реализации
Подробности подготовки и запуска описаны в [расшериенном описание решения](blog-project/readme_blog_project.md)


# Модуль 2 (клиент-серверное приложения `quote_server` и `quote_client`)


## release notes after Review 1

- добавлена поддержка аргумента `--tickers-file`;
- Вместо `--server-addr` и `--udp-port` решил обойтись одним `--target-quote-server`, который сразу парсится как сокет и добавлена ошибка ;
- клонирование tcp-стрима обернуто выполнено через match. Если клонировать поток не получится, то ошибка залогируется , а серивис продолжит работу. В TCP-поток будет отправлена ошибка.;
- Обработана очистка и регистрация клиента (match вместо unwrap);
- добавлениа fn `get_cur_timestamp()` для получения меток времени;

PS: unwrap_or для дефолтных портов решил оставить , т.к. кажется в этом случае не будет профита от match или map_err.

## Описание
Используемые крейты:
- streaming_quotes_project

Релинзная сборка проекта:
```sh 
cargo build --release -p streaming_quotes_project --bins
```
Запуск сервера:
```sh
RUST_LOG=info target/debug/quote_server
```
Запуск клиента с тикерами по умолчанию (AAPL,TSLA)
```sh
RUST_LOG=info target/release/quote_client 127.0.0.1:8001
```

![Схема приложения](./docs/home_task_2.gif)

- [Расшериенное описание решения](docs/module2/quotes_stream.md)
- [Release notes](docs/module2/release_notes.md)

# Модуль 1 (конвертер банковских выписок)
- [Описание решения](docs/module1/rr_converter_readme.md)
- [Release notes](docs/module1/release_notes.md)

Используемые крейты:
- rr-parser-lib
- rr-file-processor
