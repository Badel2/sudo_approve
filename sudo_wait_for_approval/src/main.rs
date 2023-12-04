use std::error::Error;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

fn main() -> Result<(), Box<dyn Error>> {
    // Path to the Unix domain socket
    let socket_path = "/tmp/sudo_approve.sock";

    // Attempt to connect to the Unix domain socket
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to socket: {}", e);
            return Err(e.into());
        }
    };

    // Send a message to the `sudo_approve` server
    let message = b"Hello please give me sudo";
    if let Err(e) = stream.write_all(message) {
        eprintln!("Failed to send message to the server: {}", e);
        return Err(e.into());
    }

    // Wait for a reply
    let mut response = [0; 1024];
    match stream.read(&mut response) {
        Ok(x) if x != 0 => {
            assert_ne!(x, 0);
            // Process the response here
            // For example, check if the response is a success message
            println!("Received response: {}", String::from_utf8_lossy(&response));
            // Success
        }
        Ok(x) => {
            assert_eq!(x, 0);
            eprintln!("Server closed connection, I guess you were rejected");
            return Err("connection closed".into());
        }
        Err(e) => {
            eprintln!("Failed to read response from server: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
