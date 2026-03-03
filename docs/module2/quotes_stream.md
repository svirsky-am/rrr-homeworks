
# Очистка
Убить все процессы quote_server и quote_client
```sh
pkill -f quote_server
pkill -f quote_client
```
Или более агрессивно — все процессы на портах 8000-8010
```sh
sudo lsof -ti:8000,8001 | xargs kill -9 2>/dev/null || true
```
# Debug-сборка и тесты
Сборка решения:
```sh
cargo build -p streaming_quotes_project --bins
```
запуск тестов:
```sh
cargo test -p streaming_quotes_project --test integration -- --nocapture
```
или 
```sh
make mod2-build-integerated-tests
```
# Релинзная сборка проекта:
```sh 
cargo build --release -p streaming_quotes_project --bins
```
Запуск сервера:
```sh
RUST_LOG=info target/release/quote_server
```
Запуск с тикерами по умолчанию (AAPL,TSLA)
```sh
RUST_LOG=info target/release/quote_client 127.0.0.1:8001
```
Запуск с конкретными тикерами
```sh
RUST_LOG=info ./target/release/quote_client 127.0.0.1:8001 AAPL,MSFT,NVDA
```
Запуск с одним тикером
```sh
./target/debug/quote_client 127.0.0.1:8001 TSLA
```


## debug

Запуск сервера:
```sh
RUST_LOG=info target/debug/quote_server
```
Запуск с тикерами по умолчанию (AAPL,TSLA)
```sh
RUST_LOG=info target/debug/quote_client 127.0.0.1:8001
```
Запуск с конкретными тикерами
```sh
RUST_LOG=info ./target/debug/quote_client --target-quote-server 127.0.0.1:8001 --filer-list AAPL,MSFT,NVDA
RUST_LOG=info ./target/debug/quote_client --target-quote-server 127.0.0.1:8001 --filer-file streaming_quotes_project/tests/test_quotes.lst
```

Run quote_client:
```sh 
	cargo run -p streaming_quotes_project --bin quote_client --features 'sqlite random logging'
	cargo run -p streaming_quotes_project --bin quote_client --features 'sqlite random logging' -- "127.0.0.1:8080" "1000"
```

Show open ports
```sh 
netstat -tupl 
sudo pkill quote_server
```
