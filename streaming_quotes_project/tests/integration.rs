//! Integration tests for the streaming quotes project.

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Cleans up any zombie processes on the specified ports.
fn cleanup_ports(ports: &[u16]) {
    for port in ports {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output();
        if let Ok(out) = output {
            if !out.stdout.is_empty() {
                let pids = String::from_utf8_lossy(&out.stdout);
                for pid in pids.trim().split('\n') {
                    if !pid.is_empty() {
                        let _ = Command::new("kill").args(["-9", pid]).output();
                        eprintln!("[TEST] Killed zombie process on port {}: {}", port, pid);
                    }
                }
            }
        }
    }
    thread::sleep(Duration::from_millis(200));
}

#[test]
fn test_quote_streaming_with_multiple_clients() {
    // Очистка портов перед тестом
    cleanup_ports(&[12355, 12356]);

    let build_status = Command::new("cargo")
        .args(["build", "-p", "streaming_quotes_project", "--bins"])
        .status()
        .expect("Failed to execute cargo build");
    assert!(build_status.success(), "Build failed");

    let udp_port = 12355;
    let tcp_port = udp_port + 1;
    let server_tcp_addr = format!("127.0.0.1:{}", tcp_port);

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .expect("Could not find workspace root")
        .to_path_buf();

    let target_dir = workspace_root.join("target/debug");
    let server_bin = target_dir.join("quote_server");
    let client_bin = target_dir.join("quote_client");

    eprintln!("[TEST] Target dir: {}", target_dir.display());
    assert!(
        server_bin.exists(),
        "Server binary not found: {}",
        server_bin.display()
    );
    assert!(
        client_bin.exists(),
        "Client binary not found: {}",
        client_bin.display()
    );

    eprintln!(
        "[TEST] Starting server on UDP:{} TCP:{}",
        udp_port, tcp_port
    );

    let mut run_server = Command::new(&server_bin)
        .args([&udp_port.to_string(), &tcp_port.to_string()])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    eprintln!("[TEST] Server PID: {}", run_server.id());
    thread::sleep(Duration::from_millis(1000));

    // Проверка, что сервер жив
    match run_server.try_wait() {
        Ok(Some(status)) => panic!("Server exited early: {:?}", status),
        Ok(None) => eprintln!("[TEST] Server is running"),
        Err(e) => eprintln!("[TEST] Error checking server: {}", e),
    }

    let server_stdout = run_server.stdout.take().unwrap();
    let server_stderr = run_server.stderr.take().unwrap();

    let (tx_subscribe, rx_subscribe) = mpsc::channel::<String>();
    let (tx_quote, rx_quote) = mpsc::channel::<String>();
    let (tx_timeout, rx_timeout) = mpsc::channel::<String>();

    let server_stdout_thread = thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(server_stdout);
        for line in reader.lines() {
            if let Ok(text) = line {
                eprintln!("[SERVER STDOUT] {}", text);

                // Ищем подписку
                if text.contains("CLIENT_SUBSCRIBED") {
                    let _ = tx_subscribe.send(text.clone());
                }
                // Ищем котировки (исправленный паттерн)
                if text.contains("CLIENT_QUOTE_SENT") {
                    let _ = tx_quote.send(text.clone());
                }
                // Ищем таймауты
                if text.contains("CLIENT_TIMEOUT") || text.contains("CLIENT_REMOVED") {
                    let _ = tx_timeout.send(text.clone());
                }
                // Для отладки: ищем генерацию батчей
                if text.contains("QUOTE_BATCH_GENERATED") {
                    eprintln!("[TEST DEBUG] Generator is working: {}", text);
                }
            }
        }
    });

    let _stderr_thread = thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(server_stderr);
        for line in reader.lines() {
            if let Ok(text) = line {
                eprintln!("[SERVER STDERR] {}", text);
            }
        }
    });

    let mut clients = Vec::new();
    let client_configs = [
        ("Client-1", "AAPL,TSLA"),
        ("Client-2", "MSFT,NVDA,META"),
        ("Client-3", "GOOGL,AMZN"),
        ("Client-4", "AAPL,MSFT,GOOGL"),
    ];

    for (name, tickers) in client_configs.iter() {
        eprintln!("[TEST] Starting {} subscribing to {}", name, tickers);
        let tickers_string = tickers.to_string();
        let mut run_client = Command::new(&client_bin)
            .args(["--target-quote-server", &server_tcp_addr, "--filer-list", &tickers_string])
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(&format!("Failed to start {}", name));

        if let Some(out) = run_client.stdout.take() {
            thread::spawn(move || {
                let _ = std::io::copy(&mut std::io::BufReader::new(out), &mut std::io::sink());
            });
        }
        if let Some(err) = run_client.stderr.take() {
            thread::spawn(move || {
                let _ = std::io::copy(&mut std::io::BufReader::new(err), &mut std::io::sink());
            });
        }

        clients.push((name.to_string(), run_client));
        thread::sleep(Duration::from_millis(500));
    }

    eprintln!("[TEST] Waiting for all 4 clients to subscribe...");
    for i in 0..4 {
        match rx_subscribe.recv_timeout(Duration::from_secs(10)) {
            Ok(msg) => eprintln!("[TEST] OK: Subscribed ({}): {}", i + 1, msg),
            Err(e) => {
                eprintln!(
                    "[TEST] X:FAIL Timeout waiting for subscription #{}: {:?}",
                    i + 1,
                    e
                );
                panic!("Client subscription timeout");
            }
        }
    }
    eprintln!("[TEST] Ok: All 4 clients subscribed successfully");

    eprintln!("[TEST] Waiting for quote streams...");
    let mut quote_count = 0;
    for i in 0..40 {
        match rx_quote.recv_timeout(Duration::from_secs(2)) {
            Ok(msg) => {
                eprintln!("[TEST] ✓ Quote ({}): {}", i + 1, msg);
                quote_count += 1;
                if quote_count >= 4 {
                    break;
                }
            }
            Err(_) => eprintln!("[TEST] Warning: No quote in 2-second window #{}", i + 1),
        }
    }
    assert!(
        quote_count >= 4,
        "Expected at least 4 quotes, got {}",
        quote_count
    );
    eprintln!("[TEST] OK: Received {} quotes", quote_count);

    eprintln!("[TEST] Killing Client-2 and Client-4...");
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

    eprintln!("[TEST] Waiting for client timeouts...");
    let mut timeout_count = 0;
    for _ in 0..2 {
        match rx_timeout.recv_timeout(Duration::from_secs(8)) {
            Ok(msg) => {
                eprintln!("[TEST] Ok: Timeout: {}", msg);
                timeout_count += 1;
            }
            Err(_) => panic!("Client timeout not detected"),
        }
    }
    assert_eq!(timeout_count, 2, "Expected 2 timeouts");

    thread::sleep(Duration::from_secs(2));

    eprintln!("[TEST] Cleaning up...");
    for (_, mut client) in clients {
        let _ = client.kill();
        let _ = client.wait();
    }
    let _ = run_server.kill();
    let _ = run_server.wait();
    let _ = server_stdout_thread.join();

    eprintln!("[TEST] OK: SUCCESS: Multi-client streaming test passed");
}
