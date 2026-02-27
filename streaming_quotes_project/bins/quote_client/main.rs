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

use std::env;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use streaming_quotes_project::{QuoteError, QuoteResult, StockQuote, debug, error, info};

/// Client entry point.
///
/// # Arguments
///
/// * `args[1]` - Server TCP address (e.g., "127.0.0.1:8001")
/// * `args[2]` - Comma-separated tickers (optional, default: "AAPL,TSLA")
///
/// # Returns
///
/// `Ok(())` on success, or an error that will be printed to stderr
/// by the Rust runtime with a non-zero exit code.
fn main() -> QuoteResult<()> {
    streaming_quotes_project::logging::init_logger();

    let args: Vec<String> = env::args().collect();

    // Аргумент 1: адрес сервера (обязательно)
    let server_tcp_addr: SocketAddr = args
        .get(1)
        .map(|s| s.parse::<SocketAddr>())
        .transpose()
        .map_err(|e| QuoteError::InvalidAddress(e))?
        .ok_or_else(|| QuoteError::MissingArgument("server_tcp_addr".to_string()))?;

    // Аргумент 2: тикеры (опционально)
    let tickers: Vec<String> = args
        .get(2)
        .map(|s| s.split(',').map(|t| t.trim().to_uppercase()).collect())
        .unwrap_or_else(|| vec!["AAPL".to_string(), "TSLA".to_string()]);

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
