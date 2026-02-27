use std::net::{UdpSocket, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::sync::mpsc;

fn main() {
    println!("[Server] Запуск quote_server...");

    // Привязываем сервер к порту 8000
    let socket = UdpSocket::bind("127.0.0.1:8000").expect("Не удалось привязаться к порту 8000");
    println!("[Server] Слушаем UDP на 127.0.0.1:8000");

    // Оборачиваем сокет в Arc для безопасного доступа из нескольких потоков
    let socket = Arc::new(socket);

    // Создаем канал для передачи адреса клиента от потока приема к потоку отправки
    let (tx, rx) = mpsc::channel::<SocketAddr>();

    // Клонируем сокет и передатчик для первого потока (Прием)
    let socket_recv = Arc::clone(&socket);
    let tx_clone = tx.clone();

    // --- ПОТОК 1: Прослушивание PING ---
    let listener_thread = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match socket_recv.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let message = String::from_utf8_lossy(&buf[..size]);
                    if message.trim() == "PING" {
                        println!("[Server Thread 1] Получен PING от {}", addr);
                        // Сообщаем второму потоку, куда слать ответ
                        if tx_clone.send(addr).is_err() {
                            eprintln!("[Server Thread 1] Ошибка отправки адреса в канал");
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Server Thread 1] Ошибка приема: {}", e);
                }
            }
        }
    });

    // --- ПОТОК 2: Отправка PONG ---
    let sender_thread = thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(client_addr) => {
                    println!("[Server Thread 2] Отправка PONG клиенту {}", client_addr);
                    let msg = "PONG";
                    if let Err(e) = socket.send_to(msg.as_bytes(), client_addr) {
                        eprintln!("[Server Thread 2] Ошибка отправки PONG: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("[Server Thread 2] Ошибка получения адреса из канала: {}", e);
                    break;
                }
            }
        }
    });

    // Ожидаем завершения потоков (в данном примере они работают бесконечно)
    let _ = listener_thread.join();
    let _ = sender_thread.join();
}