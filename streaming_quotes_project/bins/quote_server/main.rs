use std::net::{UdpSocket, TcpListener, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::env;
use std::io::{Write, BufRead};
use std::str::FromStr;
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Информация о клиенте
struct ClientInfo {
    tx: mpsc::Sender<()>,  // Канал для сигнала о получении PING
    last_ping: Instant,     // Время последнего PING
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let udp_port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(8000)
    } else {
        8000
    };
    
    let tcp_port = if args.len() > 2 {
        args[2].parse::<u16>().unwrap_or(8001)
    } else {
        8001
    };

    let udp_addr = format!("127.0.0.1:{}", udp_port);
    let tcp_addr = format!("127.0.0.1:{}", tcp_port);
    
    let udp_socket = UdpSocket::bind(&udp_addr).expect(&format!("Failed to bind UDP to {}", udp_addr));
    let tcp_listener = TcpListener::bind(&tcp_addr).expect(&format!("Failed to bind TCP to {}", tcp_addr));
    
    eprintln!("[SERVER] UDP listening on {}", udp_addr);
    eprintln!("[SERVER] TCP listening on {}", tcp_addr);
    let _ = std::io::stderr().flush();

    let udp_socket = Arc::new(udp_socket);
    
    // Shared: HashMap для хранения активных клиентов
    // Ключ = UDP адрес клиента, Значение = ClientInfo
    let clients: Arc<Mutex<HashMap<SocketAddr, ClientInfo>>> = Arc::new(Mutex::new(HashMap::new()));

    // === ПОТОК 1: UDP Прием PING ===
    let udp_socket_recv = Arc::clone(&udp_socket);
    let clients_clone = Arc::clone(&clients);
    let udp_listener_thread = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match udp_socket_recv.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..size]).trim().to_string();
                    eprintln!("[SERVER UDP] Received {} from {}", msg, addr);
                    let _ = std::io::stderr().flush();
                    
                    if msg == "PING" {
                        // Обновляем last_ping и отправляем сигнал в канал клиента
                        let mut clients_guard = clients_clone.lock().unwrap();
                        if let Some(client_info) = clients_guard.get_mut(&addr) {
                            client_info.last_ping = Instant::now();
                            let _ = client_info.tx.send(()); // Сигнал о получении PING
                            eprintln!("[SERVER] Notified client handler for {}", addr);
                        } else {
                            eprintln!("[SERVER] PING from unregistered client {}", addr);
                        }
                        let _ = std::io::stderr().flush();
                    }
                }
                Err(e) => eprintln!("[SERVER UDP] Recv error: {}", e),
            }
        }
    });

    // === ПОТОК 2: TCP Listener для регистрации клиентов ===
    let tcp_listener_thread = thread::spawn(move || {
        eprintln!("[SERVER TCP] Waiting for client registration...");
        let _ = std::io::stderr().flush();
        
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(tcp_stream) => {
                    let client_peer_addr = tcp_stream.peer_addr().unwrap();
                    eprintln!("[SERVER TCP] New connection from {}", client_peer_addr);
                    let _ = std::io::stderr().flush();
                    
                    // Клонируем shared данные для нового потока клиента
                    let udp_socket_clone = Arc::clone(&udp_socket);
                    let clients_clone = Arc::clone(&clients);
                    
                    // Запускаем отдельный поток для обслуживания этого клиента
                    thread::spawn(move || {
                        handle_client(tcp_stream, udp_socket_clone, clients_clone);
                    });
                }
                Err(e) => eprintln!("[SERVER TCP] Connection error: {}", e),
            }
        }
    });

    let _ = udp_listener_thread.join();
    let _ = tcp_listener_thread.join();
}

// Обработчик отдельного клиента
fn handle_client(
    mut tcp_stream: std::net::TcpStream,
    udp_socket: Arc<UdpSocket>,
    clients: Arc<Mutex<HashMap<SocketAddr, ClientInfo>>>,
) {
    // Читаем UDP адрес клиента из TCP-соединения
    let mut reader = std::io::BufReader::new(tcp_stream.try_clone().unwrap());
    let mut addr_str = String::new();
    
    match reader.read_line(&mut addr_str) {
        Ok(_) => {
            let addr_str = addr_str.trim();
            eprintln!("[SERVER TCP] Client registered UDP address: {}", addr_str);
            let _ = std::io::stderr().flush();
            
            match SocketAddr::from_str(addr_str) {
                Ok(client_udp_addr) => {
                    // Создаем канал для сигналов этому клиенту
                    let (tx, rx) = mpsc::channel::<()>();
                    
                    // Регистрируем клиента в shared HashMap
                    {
                        let mut clients_guard = clients.lock().unwrap();
                        clients_guard.insert(client_udp_addr, ClientInfo {
                            tx: tx,
                            last_ping: Instant::now(),
                        });
                        eprintln!("[SERVER] Client {} registered, {} active clients", 
                                  client_udp_addr, clients_guard.len());
                        let _ = std::io::stderr().flush();
                    }
                    
                    // Отправляем подтверждение клиенту
                    let _ = tcp_stream.write_all(b"OK\n");
                    let _ = tcp_stream.flush();
                    
                    // Отправляем первый PONG сразу после регистрации
                    eprintln!("[SERVER] Sending initial PONG to {}", client_udp_addr);
                    let _ = std::io::stderr().flush();
                    let _ = udp_socket.send_to(b"PONG", client_udp_addr);
                    
                    // === Цикл ожидания PING с таймаутом 5 секунд ===
                    let timeout = Duration::from_secs(5);
                    loop {
                        match rx.recv_timeout(timeout) {
                            Ok(()) => {
                                // Получили сигнал о новом PING - отправляем PONG
                                eprintln!("[SERVER] PING received from {}, sending PONG", client_udp_addr);
                                let _ = std::io::stderr().flush();
                                let _ = udp_socket.send_to(b"PONG", client_udp_addr);
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                // Таймаут 5 секунд без PING - завершаем поток
                                eprintln!("[SERVER] Timeout: No PING from {} for 5s, closing handler", client_udp_addr);
                                let _ = std::io::stderr().flush();
                                break;
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => {
                                // Канал закрыт (UDP приемник упал)
                                eprintln!("[SERVER] Channel disconnected for {}", client_udp_addr);
                                break;
                            }
                        }
                    }
                    
                    // Удаляем клиента из HashMap при завершении
                    {
                        let mut clients_guard = clients.lock().unwrap();
                        clients_guard.remove(&client_udp_addr);
                        eprintln!("[SERVER] Client {} removed, {} active clients", 
                                  client_udp_addr, clients_guard.len());
                        let _ = std::io::stderr().flush();
                    }
                }
                Err(e) => {
                    eprintln!("[SERVER TCP] Invalid address format: {}", e);
                    let _ = std::io::stderr().flush();
                    let _ = tcp_stream.write_all(b"ERROR: Invalid address\n");
                }
            }
        }
        Err(e) => {
            eprintln!("[SERVER TCP] Read error: {}", e);
            let _ = std::io::stderr().flush();
        }
    }
}