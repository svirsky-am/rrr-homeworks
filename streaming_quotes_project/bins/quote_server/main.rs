//! Quote server binary.
//!
//! Starts a TCP/UDP server that accepts client subscriptions and streams stock quotes.
//!
//! # Architecture
//!
//! - One generator thread produces quotes for all tickers every 2 seconds
//! - `QuoteBroadcaster` fans out batches to per-client sender threads via `mpsc`
//! - Each client sender filters by subscription and sends via UDP
//! - UDP PING/PONG handled in separate thread
//! - Timeout monitor terminates inactive clients after 5 seconds

use std::collections::HashMap;
use std::env;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use streaming_quotes_project::{
    QuoteBatch, QuoteError, QuoteGenerator, QuoteResult, StockQuote, debug, error, info,
    is_supported_ticker, warn,
};

/// Type alias for quote channel sender.
type QuoteSender = mpsc::Sender<QuoteBatch>;

/// Broadcasts quote batches to multiple client receivers via mpsc fan-out.
///
/// Holds a vector of senders; `broadcast()` sends a clone of the batch
/// to each registered receiver. Dead receivers are automatically removed.
#[derive(Clone)]
struct QuoteBroadcaster {
    senders: Arc<Mutex<Vec<QuoteSender>>>,
}

impl QuoteBroadcaster {
    /// Creates a new empty broadcaster.
    fn new() -> Self {
        Self {
            senders: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registers a new client and returns its receiver.
    ///
    /// The returned receiver will receive all future broadcasts.
    fn register(&self) -> mpsc::Receiver<QuoteBatch> {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut senders) = self.senders.lock() {
            senders.push(tx);
        }
        rx
    }

    /// Broadcasts a quote batch to all registered clients.
    ///
    /// Dead channels (disconnected receivers) are removed from the list.
    fn broadcast(&self, batch: QuoteBatch) {
        if let Ok(mut senders) = self.senders.lock() {
            senders.retain(|tx| tx.send(batch.clone()).is_ok());
        }
    }
}

/// Client session information.
struct ClientSession {
    udp_addr: SocketAddr,
    tickers: Vec<String>,
    last_activity: Instant,
    stop_flag: Arc<AtomicBool>,
}

fn main() -> QuoteResult<()> {
    streaming_quotes_project::logging::init_logger();

    let args: Vec<String> = env::args().collect();

    let udp_port: u16 = args
        .get(1)
        .map(|s| s.parse::<u16>())
        .transpose()
        .map_err(|e| QuoteError::ArgumentError(format!("Invalid UDP port: {}", e)))?
        .unwrap_or(8000);

    let tcp_port: u16 = args
        .get(2)
        .map(|s| s.parse::<u16>())
        .transpose()
        .map_err(|e| QuoteError::ArgumentError(format!("Invalid TCP port: {}", e)))?
        .unwrap_or(8001);

    let udp_addr = format!("127.0.0.1:{}", udp_port);
    let tcp_addr = format!("127.0.0.1:{}", tcp_port);

    let udp_socket = Arc::new(
        UdpSocket::bind(&udp_addr).map_err(|e| QuoteError::BindError {
            addr: udp_addr.clone(),
            source: e,
        })?,
    );

    let tcp_listener = TcpListener::bind(&tcp_addr).map_err(|e| QuoteError::BindError {
        addr: tcp_addr.clone(),
        source: e,
    })?;

    info!("SERVER_STARTED udp={} tcp={}", udp_addr, tcp_addr);

    let broadcaster = Arc::new(QuoteBroadcaster::new());

    let clients: Arc<Mutex<HashMap<SocketAddr, ClientSession>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Thread 1: Quote Generator (ALL tickers, every 2 seconds)
    let gen_broadcaster = Arc::clone(&broadcaster);
    let gen_thread = thread::spawn(move || {
        run_quote_generator(gen_broadcaster);
    });

    // Thread 2: UDP Ping/Pong listener
    let udp_ping = Arc::clone(&udp_socket);
    let clients_ping = Arc::clone(&clients);
    let ping_thread = thread::spawn(move || {
        run_ping_listener(udp_ping, clients_ping);
    });

    // Thread 3: TCP Listener for subscriptions
    let clients_tcp = Arc::clone(&clients);
    let udp_tcp = Arc::clone(&udp_socket);
    let broadcaster_tcp = Arc::clone(&broadcaster);
    let tcp_thread = thread::spawn(move || {
        run_tcp_listener(tcp_listener, udp_tcp, clients_tcp, broadcaster_tcp);
    });

    // Thread 4: Timeout monitor
    let clients_monitor = Arc::clone(&clients);
    let monitor_thread = thread::spawn(move || {
        run_timeout_monitor(clients_monitor);
    });

    // Wait for all threads (they run forever in this implementation)
    let _ = gen_thread.join();
    let _ = ping_thread.join();
    let _ = tcp_thread.join();
    let _ = monitor_thread.join();

    Ok(())
}

/// Generates quotes for ALL supported tickers every 2 seconds.
///
/// Broadcasts each batch to all registered client receivers.
fn run_quote_generator(broadcaster: Arc<QuoteBroadcaster>) {
    info!("QUOTE_GENERATOR_STARTED interval=2s");

    let mut generator = QuoteGenerator::new();
    let interval = Duration::from_secs(2);

    loop {
        let batch = generator.generate_all_quotes();
        info!("QUOTE_BATCH_GENERATED count={}", batch.quotes.len());

        broadcaster.broadcast(batch);

        thread::sleep(interval);
    }
}

/// Runs the UDP ping/pong listener.
///
/// Updates client activity timestamp on PING to prevent timeout.
fn run_ping_listener(
    udp_socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashMap<SocketAddr, ClientSession>>>,
) {
    let mut buf = [0u8; 1024];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]).trim().to_string();
                if msg == "PING" {
                    debug!("UDP_PING from {}", addr);

                    if let Ok(mut clients_guard) = clients.lock() {
                        if let Some(session) = clients_guard.get_mut(&addr) {
                            session.last_activity = Instant::now();
                            let _ = udp_socket.send_to(b"PONG", addr);
                            debug!("UDP_PONG to {}", addr);
                        }
                    }
                }
            }
            Err(e) => error!("UDP_RECV_ERROR: {}", e),
        }
    }
}

/// Runs the TCP listener for client subscriptions.
///
/// Spawns a handler thread for each new connection.
fn run_tcp_listener(
    listener: TcpListener,
    udp_socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashMap<SocketAddr, ClientSession>>>,
    broadcaster: Arc<QuoteBroadcaster>,
) {
    info!("TCP_LISTENER_READY");

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                let client_addr = match tcp_stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(e) => {
                        error!("Failed to get peer address: {}", e);
                        continue;
                    }
                };
                debug!("TCP_NEW_CONNECTION from {}", client_addr);

                let udp_clone = Arc::clone(&udp_socket);
                let clients_clone = Arc::clone(&clients);
                let broadcaster_clone = Arc::clone(&broadcaster);

                thread::spawn(move || {
                    if let Err(e) = handle_client_subscription(
                        tcp_stream,
                        udp_clone,
                        clients_clone,
                        broadcaster_clone,
                    ) {
                        error!("Client handler error: {}", e);
                    }
                });
            }
            Err(e) => error!("TCP_CONNECTION_ERROR: {}", e),
        }
    }
}

/// Handles a client subscription.
///
/// 1. Parses STREAM command
/// 2. Registers client session
/// 3. Creates per-client receiver from broadcaster
/// 4. Spawns UDP sender thread for this client
/// 5. Waits for client to disconnect or timeout
fn handle_client_subscription(
    mut tcp_stream: std::net::TcpStream,
    udp_socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashMap<SocketAddr, ClientSession>>>,
    broadcaster: Arc<QuoteBroadcaster>,
) -> QuoteResult<()> {
    let mut reader = std::io::BufReader::new(tcp_stream.try_clone().unwrap());
    let mut command = String::new();

    reader
        .read_line(&mut command)
        .map_err(|e| QuoteError::SendError(e))?;

    let command = command.trim();
    debug!("TCP_COMMAND_RECEIVED: {}", command);

    let session = parse_stream_command(command)
        .ok_or_else(|| QuoteError::InvalidCommand(command.to_string()))?;

    let client_udp_addr = session.udp_addr;
    let tickers = session.tickers.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Register client in shared registry
    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.insert(
            client_udp_addr,
            ClientSession {
                udp_addr: client_udp_addr,
                tickers: tickers.clone(),
                last_activity: Instant::now(),
                stop_flag: Arc::clone(&stop_flag),
            },
        );
        info!(
            "CLIENT_SUBSCRIBED addr={} tickers={:?}",
            client_udp_addr, tickers
        );
    }

    // Send confirmation to client
    tcp_stream.write_all(b"OK\n")?;
    tcp_stream.flush()?;

    // Register this client with the broadcaster to receive quote batches
    let client_rx = broadcaster.register();

    // Spawn the client's UDP sender thread
    let sender_stop_flag = Arc::clone(&stop_flag);
    let sender_udp = Arc::clone(&udp_socket);
    let sender_tickers = tickers.clone();
    let sender_addr = client_udp_addr;

    let sender_thread = thread::spawn(move || {
        run_client_sender(
            client_rx,
            sender_udp,
            sender_addr,
            sender_tickers,
            sender_stop_flag,
        );
    });

    // Wait for the sender thread to complete (on timeout or disconnect)
    let _ = sender_thread.join();

    // Clean up client session
    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.remove(&client_udp_addr);
        debug!("CLIENT_SESSION_ENDED addr={}", client_udp_addr);
    }

    Ok(())
}

/// Runs the UDP sender thread for a single client.
///
/// Receives QuoteBatch from generator, filters by subscribed tickers,
/// and sends via UDP. Exits when stop_flag is set or after inactivity.
fn run_client_sender(
    rx: mpsc::Receiver<QuoteBatch>,
    udp_socket: Arc<UdpSocket>,
    client_addr: SocketAddr,
    subscribed_tickers: Vec<String>,
    stop_flag: Arc<AtomicBool>,
) {
    info!(
        "CLIENT_SENDER_STARTED addr={} tickers={:?}",
        client_addr, subscribed_tickers
    );

    let timeout = Duration::from_secs(5);

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            info!("CLIENT_SENDER_STOPPED addr={} (timeout flag)", client_addr);
            break;
        }
        match rx.recv_timeout(timeout) {
            Ok(batch) => {
                let filtered: Vec<StockQuote> = batch
                    .quotes
                    .iter()
                    .filter(|q| subscribed_tickers.contains(&q.ticker))
                    .cloned()
                    .collect::<Vec<_>>();

                // Send each filtered quote via UDP
                for quote in filtered {
                    let data = quote.to_bytes();
                    if let Err(e) = udp_socket.send_to(&data, client_addr) {
                        error!("CLIENT_SEND_ERROR addr={} error={}", client_addr, e);
                        break;
                    }
                    info!(
                        "CLIENT_QUOTE_SENT addr={} ticker={}",
                        client_addr, quote.ticker
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                debug!("CLIENT_SENDER_RECV_TIMEOUT addr={}", client_addr);
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                warn!(
                    "CLIENT_SENDER_DISCONNECTED addr={} (channel closed)",
                    client_addr
                );
                break;
            }
        }
    }

    info!("CLIENT_SENDER_FINISHED addr={}", client_addr);
}

/// Runs the timeout monitor.
///
/// Checks client activity every 2 seconds and terminates inactive clients
/// by setting their stop_flag after 5 seconds of inactivity.
fn run_timeout_monitor(clients: Arc<Mutex<HashMap<SocketAddr, ClientSession>>>) {
    loop {
        thread::sleep(Duration::from_secs(2));

        let mut to_remove = Vec::new();
        {
            if let Ok(clients_guard) = clients.lock() {
                for (addr, session) in clients_guard.iter() {
                    if session.last_activity.elapsed() > Duration::from_secs(5) {
                        warn!("CLIENT_TIMEOUT addr={}", addr);
                        to_remove.push(*addr);
                    }
                }
            }
        }

        for addr in to_remove {
            if let Ok(mut clients_guard) = clients.lock() {
                if let Some(session) = clients_guard.remove(&addr) {
                    session.stop_flag.store(true, Ordering::Relaxed);
                    info!("CLIENT_REMOVED addr={}", addr);
                }
            }
        }
    }
}

/// Parses a STREAM command from client.
///
/// Expected format: `STREAM udp://<addr> <ticker1>,<ticker2>,...`
///
/// # Returns
///
/// `Some(ClientSession)` if parsing succeeds, `None` otherwise.
fn parse_stream_command(cmd: &str) -> Option<ClientSession> {
    if !cmd.starts_with("STREAM ") {
        return None;
    }

    let rest = &cmd[7..];
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let udp_part = parts[0];
    if !udp_part.starts_with("udp://") {
        return None;
    }
    let udp_addr_str = &udp_part[6..];
    let udp_addr = udp_addr_str.parse::<SocketAddr>().ok()?;

    let tickers: Vec<String> = parts[1]
        .split(',')
        .map(|t| t.trim().to_uppercase())
        .filter(|t| is_supported_ticker(t))
        .collect();

    if tickers.is_empty() {
        return None;
    }

    Some(ClientSession {
        udp_addr,
        tickers,
        last_activity: Instant::now(),
        stop_flag: Arc::new(AtomicBool::new(false)),
    })
}
