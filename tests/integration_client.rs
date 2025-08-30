use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

// Use a different port for testing to avoid conflicts
const TEST_PORT: u16 = 8999;

async fn wait_for_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://localhost:{port}/");
    println!("Waiting for server to be ready at {url}");

    // Try to connect to the health endpoint until it responds or timeout
    for attempt in 0..40 {
        println!("Attempt {} to connect to server", attempt + 1);
        match reqwest::get(&url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    println!("Server is ready!");
                    return Ok(());
                } else {
                    println!("Server responded with status: {}", resp.status());
                }
            }
            Err(e) => {
                println!("Connection attempt {} failed: {}", attempt + 1, e);
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Err("Server did not start in time".into())
}

#[tokio::test]
#[ignore] // Skip this test in CI environments due to server startup timeouts
async fn test_client_endpoints() {
    // Spawn the client server as a subprocess
    let mut cmd = Command::new("cargo");
    let child = cmd
        .args(["run", "--", "client", "--port", &TEST_PORT.to_string(), "--cid", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start client server");

    // Give the server a moment to start up
    sleep(Duration::from_millis(1000)).await;

    // Wait for the server to be ready
    if let Err(e) = wait_for_server(TEST_PORT).await {
        // If server didn't start, print stderr for debugging
        let output = child.wait_with_output().expect("Failed to wait for child process");
        println!("Server stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Server stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Server failed to start: {e}");
    }

    // Test health check endpoint
    let url = format!("http://localhost:{TEST_PORT}/");
    let resp = reqwest::get(&url)
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read body");
    assert_eq!(body, "\"Hola!!!\"");

    // Test get-keys endpoint
    let keys_url = format!("http://localhost:{TEST_PORT}/get-keys/");
    let resp = reqwest::get(&keys_url)
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    let json: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(json.get("private_key").is_some());
    assert!(json.get("public_key_x").is_some());
    assert!(json.get("public_key_y").is_some());

    // Kill the server process gracefully
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
        // Give it a moment to shut down gracefully
        sleep(Duration::from_millis(100)).await;
        // Force kill if still running
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGKILL);
    }
    #[cfg(windows)]
    {
        let _ = child.kill();
    }

    // Wait for the process to exit
    match child.wait_with_output() {
        Ok(output) => {
            println!("Server exited with status: {}", output.status);
            if !output.status.success() {
                println!("Server stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("Failed to wait for server process: {e}");
        }
    }
}
