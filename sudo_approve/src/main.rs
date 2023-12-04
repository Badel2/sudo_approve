use std::error::Error;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::mpsc;

async fn handle_client(mut stream: tokio::net::UnixStream, tx: mpsc::Sender<String>, now: Instant) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
        Ok(size) => {
            let request_str = String::from_utf8_lossy(&buffer[..size]).to_string();
            tx.send(request_str).await.unwrap();
            // TODO: this should wait for the TUI to mark the request as valid
            print!("Approve? [y/N] ");
            let _ = io::stdout().flush();
            let mut buffer = String::new();
            let response = {
                // Start new scope to avoid keeping stdin across await points
                let stdin = io::stdin();
                stdin.read_line(&mut buffer).unwrap();

                buffer.trim()
            };
            let response_time = now.elapsed();
            let ten_seconds = Duration::from_secs(10);
            if response_time > ten_seconds {
                println!(
                    "Rejected because of timeout, this request has been alive for {:?}",
                    response_time
                );
                return;
            }
            if response == "y" || response == "Y" {
                stream.write_all(response.as_bytes()).await.unwrap();
                println!("Approved");
            } else {
                println!("Rejected");
            }
        }
        Err(e) => {
            eprintln!("Failed to read from the socket: {}", e);
        }
    }
}

async fn process_requests(mut rx: mpsc::Receiver<String>) {
    while let Some(_request) = rx.recv().await {
        //println!("Processing request: {}", request);
        // TODO: this should update the TUI
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket_path = "/tmp/sudo_approve.sock";

    // TODO: if this fails, delete file and try again
    // or detect that an instance of this process is already running
    // TODO: this is not secure because if the socket_path does not exist already, anyone can
    // create it and bind to it. Should be enough to change /tmp for a folder that can only be
    // created by root
    // TODO: add a check to ensure this script is running as root?
    let listener = UnixListener::bind(socket_path)?;

    // Allow normal users to connect
    let mut perms = std::fs::metadata(socket_path)?.permissions();
    perms.set_mode(0o777);
    std::fs::set_permissions(socket_path, perms)?;

    // Channel for processing logic communication
    let (tx, rx) = mpsc::channel(32);

    // Spawn a task for request processing
    tokio::spawn(async move {
        process_requests(rx).await;
    });

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let now_i = Instant::now();
                let now = chrono::offset::Local::now();
                println!("Got connection at {:?} from {:?}", now, addr);
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    handle_client(stream, tx_clone, now_i).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                break;
            }
        }
    }

    Ok(())
}
