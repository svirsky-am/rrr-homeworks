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
