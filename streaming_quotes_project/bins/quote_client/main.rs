use std::net::{UdpSocket, TcpStream, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::env;
use std::io::{Write, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let server_tcp_addr: SocketAddr = if args.len() > 1 {
        args[1].parse().expect("Invalid server TCP address")
    } else {
        "127.0.0.1:8001".parse().unwrap()
    };

    let interval_ms = if args.len() > 2 {
        args[2].parse().unwrap_or(1000)
    } else {
        1000
    };

    let udp_socket = UdpSocket::bind("127.0.0.1:0").expect("Cannot bind client UDP socket");
    let client_udp_addr = udp_socket.local_addr().expect("Cannot get local UDP address");
    
    eprintln!("[CLIENT] Local UDP address: {}", client_udp_addr);
    eprintln!("[CLIENT] Connecting to server TCP: {}", server_tcp_addr);
    let _ = std::io::stderr().flush();

    // TCP-регистрация
    let mut tcp_stream = TcpStream::connect(server_tcp_addr).expect("Failed to connect to server TCP");
    eprintln!("[CLIENT] TCP connected, sending UDP address...");
    let _ = std::io::stderr().flush();
    
    let addr_msg = format!("{}\n", client_udp_addr);
    tcp_stream.write_all(addr_msg.as_bytes()).expect("Failed to send UDP address");
    tcp_stream.flush().expect("Failed to flush TCP");
    
    let mut reader = std::io::BufReader::new(tcp_stream.try_clone().unwrap());
    let mut response = String::new();
    reader.read_line(&mut response).expect("Failed to read server response");
    eprintln!("[CLIENT] Server response: {}", response.trim());
    let _ = std::io::stderr().flush();

    let udp_socket = Arc::new(udp_socket);
    let udp_socket_send = Arc::clone(&udp_socket);

    // UDP порт сервера = TCP порт - 1
    let server_udp_addr: SocketAddr = format!("127.0.0.1:{}", server_tcp_addr.port() - 1)
        .parse().expect("Failed to parse server UDP address");

    // ПОТОК 1: Отправка PING
    let sender_thread = thread::spawn(move || {
        let msg = "PING";
        loop {
            match udp_socket_send.send_to(msg.as_bytes(), server_udp_addr) {
                Ok(size) => {
                    eprintln!("[CLIENT UDP] Sent PING ({} bytes)", size);
                    let _ = std::io::stderr().flush();
                }
                Err(e) => eprintln!("[CLIENT UDP] Send error: {}", e),
            }
            thread::sleep(Duration::from_millis(interval_ms));
        }
    });

    // ПОТОК 2: Получение PONG
    let receiver_thread = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match udp_socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    if msg.trim() == "PONG" {
                        eprintln!("[CLIENT UDP] Received PONG from {}", addr);
                        let _ = std::io::stderr().flush();
                    }
                }
                Err(e) => eprintln!("[CLIENT UDP] Recv error: {}", e),
            }
        }
    });

    let _ = sender_thread.join();
    let _ = receiver_thread.join();
}