use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use std::env;

#[test]
fn test_udp_quote_client_server_with_multiple_clients() {
    // 1. Собираем бинарники
    let build_status = Command::new("cargo")
        .args(["build", "-p", "streaming_quotes_project", "--bins"])
        .status()
        .expect("Failed to execute cargo build");
    assert!(build_status.success(), "Build failed");

    let udp_port = 12352;
    let tcp_port = udp_port + 1;
    let server_tcp_addr = format!("127.0.0.1:{}", tcp_port);
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("Could not find workspace root")
        .to_path_buf();
    
    let target_dir = workspace_root.join("target/debug");
    let server_bin = target_dir.join("quote_server");
    let client_bin = target_dir.join("quote_client");

    eprintln!("[TEST] Target dir: {}", target_dir.display());
    assert!(server_bin.exists(), "Server binary not found: {}", server_bin.display());
    assert!(client_bin.exists(), "Client binary not found: {}", client_bin.display());

    eprintln!("[TEST] Starting server on UDP:{} TCP:{}", udp_port, tcp_port);

    // 2. Запускаем сервер
    let mut run_server = Command::new(&server_bin)
        .args([&udp_port.to_string(), &tcp_port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    thread::sleep(Duration::from_millis(500));

    // 3. Читаем stderr сервера
    let server_stderr = run_server.stderr.take().unwrap();
    
    let (tx_register, rx_register) = mpsc::channel::<String>();
    let (tx_timeout, rx_timeout) = mpsc::channel::<String>();

    let server_stderr_thread = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(server_stderr);
        for line in std::io::BufRead::lines(reader) {
            if let Ok(text) = line {
                eprintln!("[SERVER LOG] {}", text);
                
                if text.contains("Client registered UDP address") {
                    let _ = tx_register.send(text.clone());
                }
                
                if text.contains("Timeout: No PING") || text.contains("closing handler") {
                    let _ = tx_timeout.send(text.clone());
                }
            }
        }
    });

    if let Some(stdout) = run_server.stdout.take() {
        std::thread::spawn(move || {
            let _ = std::io::copy(&mut std::io::BufReader::new(stdout), &mut std::io::sink());
        });
    }

    thread::sleep(Duration::from_millis(300));

    // 4. Запускаем 4 клиента
    let mut clients = Vec::new();
    let client_configs = [
        ("Client-1", 500),
        ("Client-2", 1000),
        ("Client-3", 800),
        ("Client-4", 600),
    ];

    for (name, interval) in client_configs.iter() {
        eprintln!("[TEST] Starting {} with interval {}ms", name, interval);
        let mut run_client = Command::new(&client_bin)
            .args([&server_tcp_addr, &interval.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(&format!("Failed to start {}", name));

        if let Some(out) = run_client.stdout.take() {
            std::thread::spawn(move || {
                let _ = std::io::copy(&mut std::io::BufReader::new(out), &mut std::io::sink());
            });
        }
        if let Some(err) = run_client.stderr.take() {
            std::thread::spawn(move || {
                let _ = std::io::copy(&mut std::io::BufReader::new(err), &mut std::io::sink());
            });
        }

        clients.push((name.to_string(), run_client));
        thread::sleep(Duration::from_millis(200));
    }

    // 5. Проверяем регистрацию всех 4 клиентов
    eprintln!("[TEST] Waiting for all 4 clients to register...");
    let mut registered_count = 0;
    for _ in 0..4 {
        match rx_register.recv_timeout(Duration::from_secs(5)) {
            Ok(msg) => {
                eprintln!("[TEST] ✓ Registered: {}", msg);
                registered_count += 1;
            }
            Err(_) => panic!("[TEST] ✗ Client registration timeout"),
        }
    }
    assert_eq!(registered_count, 4, "Not all clients registered");
    eprintln!("[TEST] ✓ All 4 clients registered successfully");

    // 6. Проверяем, что сервер видит 4 активных клиента
    thread::sleep(Duration::from_millis(500));

    // 7. ИСПРАВЛЕНО: Убиваем 2 клиента без конфликта заиммования
    eprintln!("[TEST] Killing Client-2 and Client-4, waiting for server timeout...");
    
    let mut remaining_clients = Vec::new();

    for (name, mut client) in clients {
        if name == "Client-2" || name == "Client-4" {
            let _ = client.kill();
            let _ = client.wait();
            eprintln!("[TEST] Killed {}", name);
        } else {
            remaining_clients.push((name, client));
        }
    }

    clients = remaining_clients;

    // 8. Ждем таймауты от сервера
    eprintln!("[TEST] Waiting for 2 client timeouts...");
    let mut timeout_count = 0;
    for _ in 0..2 {
        match rx_timeout.recv_timeout(Duration::from_secs(7)) {
            Ok(msg) => {
                eprintln!("[TEST] Timeout detected: {}", msg);
                timeout_count += 1;
            }
            Err(_) => panic!("[TEST] Client timeout not detected within 7 seconds"),
        }
    }
    assert_eq!(timeout_count, 2, "Not all killed clients triggered timeout");

    // 9. Убиваем оставшихся клиентов
    eprintln!("[TEST] Killing remaining clients...");
    for (_, mut client) in clients {
        let _ = client.kill();
        let _ = client.wait();
    }

    // 10. Завершаем сервер
    let _ = run_server.kill();
    let _ = run_server.wait();
    let _ = server_stderr_thread.join();
    
    eprintln!("[TEST] ✓ SUCCESS: Server handled 4 clients correctly");
    eprintln!("[TEST] ✓ {} clients registered", registered_count);
    eprintln!("[TEST] ✓ {} clients timed out correctly", timeout_count);
}