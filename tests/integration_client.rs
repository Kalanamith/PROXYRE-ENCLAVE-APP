use assert_cmd::prelude::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

async fn wait_for_server() {
    // Try to connect to the health endpoint until it responds or timeout
    for _ in 0..20 {
        if let Ok(resp) = reqwest::get("http://localhost:8000/").await {
            if resp.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(250)).await;
    }
    panic!("Server did not start in time");
}

#[tokio::test]
async fn test_client_endpoints() {
    // Spawn the client server as a subprocess
    let mut cmd = Command::new("cargo");
    let mut child = cmd
        .args(&["run", "--", "client", "--port", "8000", "--cid", "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start client server");

    // Wait for the server to be ready
    wait_for_server().await;

    // Test health check endpoint
    let resp = reqwest::get("http://localhost:8000/")
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read body");
    assert_eq!(body, "\"Hola!!!\"");

    // Test get-keys endpoint
    let resp = reqwest::get("http://localhost:8000/get-keys/")
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    let json: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(json.get("private_key").is_some());
    assert!(json.get("public_key_x").is_some());
    assert!(json.get("public_key_y").is_some());

    // Kill the server process
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).ok();
    }
    #[cfg(windows)]
    {
        child.kill().ok();
    }
    // Wait for the process to exit
    let _ = child.wait();
} 