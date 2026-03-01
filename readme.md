# Модуль 2
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

Используемые крейты:
- streaming_quotes_project

# Модуль 1
- [Описание решения](docs/module1/rr_converter_readme.md)
- [Release notes](docs/module1/release_notes.md)

Используемые крейты:
- rr-parser-lib
- rr-file-processor
