//! Quote client binary.
//!
//! Connects to a quote server and receives stock quote streams.
//!
//! # Usage
//!
//! ```bash
//! # Connect to server with default tickers (AAPL,TSLA)
//! ./quote_client 127.0.0.1:8001
//!
//! # Connect to server with specific tickers
//! ./quote_client 127.0.0.1:8001 AAPL,MSFT,NVDA
//! ```

use clap::{Arg, ArgGroup, Command};
use core::clone::Clone;
use core::option::Option::{None, Some};
use std::env;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use streaming_quotes_project::{QuoteError, QuoteResult, StockQuote, debug, error, info};

pub struct Cli {
    pub target_quote_server: String,
    pub tickers_list: Vec<String>,
}

fn parse_cli() -> Result<Cli, Box<dyn std::error::Error>> {
    let matches = Command::new("quote_client")
        .version("0.1.0")
        .about("Client of quote streamer")
        .group(
            ArgGroup::new("filter")
                .args(["filer-list", "filer-file"])
                .required(true)
                .multiple(false),
        )
        .arg(
            Arg::new("target-quote-server")
                .short('i')
                .long("target-quote-server")
                .help("Target quote server with port like: 192.168.8.8:8800")
                .required(true)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("filer-list")
                .short('l')
                .long("filer-list")
                .help("Filter for published quotes as string list. Looks like: \"MSFT,NVDA,META\"")
                // .default_value("-")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("filer-file")
                .short('f')
                .long("filer-file")
                .help("Filter from file, one ticker per line")
                .value_parser(clap::value_parser!(String)),
        )
        .get_matches();

    let tickers_filter_list = matches.get_one::<String>("filer-list");
    let tickers_filter_file = matches.get_one::<String>("filer-file");

    let tickers_list: Result<Vec<String>, QuoteError> =
        match (tickers_filter_list, tickers_filter_file) {
            (filer_list, None) => {
                let arg_str = filer_list.unwrap().clone();
                Ok(arg_str
                    .split(',')
                    .map(|t| t.trim().to_uppercase())
                    .collect())
            }
            (None, filer_file) => {
                let arg_str = filer_file.unwrap().clone();
                let content = std::fs::read_to_string(Path::new(&arg_str))?;
                Ok(content
                    .lines()
                    .map(|t| t.trim().to_uppercase())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>())
            }
            (_filer_list, _filer_file) => Err(QuoteError::BothFiltersProvided), // просто для полноты
            _ => Err(QuoteError::MissingFilterArgument),
        };

    Ok(Cli {
        target_quote_server: matches
            .get_one::<String>("target-quote-server")
            .unwrap()
            .clone(),
        tickers_list: tickers_list?,
    })
}

/// Client entry point.
///
/// # Arguments
///
/// * `--target-quote-server` - Server TCP address (e.g., "127.0.0.1:8001")
/// * --filer_list - Comma-separated tickers (optional, default: "AAPL,TSLA")
///
/// # Returns
///
/// `Ok(())` on success, or an error that will be printed to stderr
/// by the Rust runtime with a non-zero exit code.
fn main() -> QuoteResult<()> {
    let cli = parse_cli().unwrap();
    // let process_input_type = cli.target_quote_server;
    // let process_output_type = cli.filer_list;

    streaming_quotes_project::logging::init_logger();

    // let mut tickers: Vec<String> = Vec::new();

    let tickers = cli.tickers_list;

    // let args: Vec<String> = env::args().collect();

    // Аргумент 1: адрес сервера (обязательно)
    let server_tcp_addr: SocketAddr = cli
        .target_quote_server
        .parse::<SocketAddr>()
        .map_err(|e| QuoteError::InvalidAddress(e))?;
    // .get(1)
    // .map(|s| s.parse::<SocketAddr>())
    // .transpose()
    // .map_err(|e| QuoteError::InvalidAddress(e))?
    // .ok_or_else(|| QuoteError::MissingArgument("server_tcp_addr".to_string()))?;

    // Аргумент 2: тикеры (опционально)
    // let tickers: Vec<String> = args
    //     .get(2)
    //     .map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect())
    //     .unwrap_or_else(|| vec!["AAPL".to_string(), "TSLA".to_string()]);

    let client_udp_bind: SocketAddr = "127.0.0.1:0"
        .parse::<SocketAddr>()
        .map_err(|e| QuoteError::InvalidAddress(e))?;

    let udp_socket = UdpSocket::bind(client_udp_bind).map_err(|e| QuoteError::BindError {
        addr: client_udp_bind.to_string(),
        source: e,
    })?;

    let client_udp_addr = udp_socket.local_addr().map_err(|e| QuoteError::BindError {
        addr: "local".to_string(),
        source: e,
    })?;

    info!(
        "CLIENT_STARTED udp={} tickers={:?}",
        client_udp_addr, tickers
    );
    info!("CLIENT_CONNECTING tcp={}", server_tcp_addr);

    let mut tcp_stream =
        TcpStream::connect(server_tcp_addr).map_err(|e| QuoteError::ConnectError {
            addr: server_tcp_addr.to_string(),
            source: e,
        })?;

    let tickers_str = tickers.join(",");
    let stream_cmd = format!("STREAM udp://{} {}\n", client_udp_addr, tickers_str);
    debug!("CLIENT_SENDING_COMMAND: {}", stream_cmd.trim());

    tcp_stream.write_all(stream_cmd.as_bytes())?;
    tcp_stream.flush()?;

    let mut reader = std::io::BufReader::new(tcp_stream.try_clone().unwrap());
    let mut response = String::new();
    reader.read_line(&mut response)?;
    info!("CLIENT_SERVER_RESPONSE: {}", response.trim());

    let udp_socket = Arc::new(udp_socket);

    // Поток 1: Получение котировок
    let receiver_thread = thread::spawn({
        let socket = Arc::clone(&udp_socket);
        move || {
            run_quote_receiver(socket);
        }
    });

    // Поток 2: PING keep-alive
    let ping_thread = thread::spawn({
        let socket = Arc::clone(&udp_socket);
        let server_udp_addr: SocketAddr = format!("127.0.0.1:{}", server_tcp_addr.port() - 1)
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:8000".parse().unwrap());
        move || {
            run_ping_sender(socket, server_udp_addr);
        }
    });

    let _ = receiver_thread.join();
    let _ = ping_thread.join();

    Ok(())
}

/// Runs the quote receiver loop.
fn run_quote_receiver(socket: Arc<UdpSocket>) {
    let mut buf = [0u8; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                if let Some(quote) = StockQuote::from_string(&msg) {
                    info!(
                        "CLIENT_QUOTE_RECEIVED ticker={} price={} volume={}",
                        quote.ticker, quote.price, quote.volume
                    );
                }
            }
            Err(e) => error!("CLIENT_UDP_RECV_ERROR: {}", e),
        }
    }
}

/// Runs the ping sender loop.
fn run_ping_sender(socket: Arc<UdpSocket>, server_addr: SocketAddr) {
    loop {
        if let Err(e) = socket.send_to(b"PING", server_addr) {
            error!("CLIENT_PING_ERROR: {}", e);
        }
        thread::sleep(Duration::from_secs(2));
    }
}
