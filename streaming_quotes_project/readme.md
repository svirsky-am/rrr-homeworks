
Run quote_server:
```sh 
	cargo run -p streaming_quotes_project --bin quote_server --features 'sqlite random logging'
```

Run quote_client:
```sh 
	cargo run -p streaming_quotes_project --bin quote_client --features 'sqlite random logging'
	cargo run -p streaming_quotes_project --bin quote_client --features 'sqlite random logging' -- "127.0.0.1:8080" "1000"
	        #     "--",
            # "127.0.0.1:8080",
            # "1000"
```

## debug
Show open ports
```sh 
netstat -tupl 
sudo pkill quote_server

```