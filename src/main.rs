use web_server::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::io::BufReader;
use std::thread;
use std::time::Duration;

fn main() {
    // Bind the server to a local address and port (127.0.0.1:7878).
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    
    // Create a thread pool with a specified number of threads (10 in this case).
    // This allows the server to handle multiple connections concurrently.
    let pool = ThreadPool::new(10);

    // Only accept and process the first 5 incoming connections before shutting down.
    for stream in listener.incoming().take(5) {
        // Unwrap the stream to handle any errors immediately.
        let stream = stream.unwrap();

        // Use the thread pool to handle each connection concurrently.
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Server shutting down.");
}

// This function handles individual connections to the server.
fn handle_connection(mut stream: TcpStream) {
    // Create a buffered reader for the incoming stream.
    let buf_reader = BufReader::new(&mut stream);

    // Read the first line of the HTTP request.
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // Define HTTP response based on the request received.
    // Serve the home page for "GET /" requests, a sleep page for "GET /sleep",
    // and return a 404 error page for any other requests.
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            // Simulate a delay in response to demonstrate thread handling.
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }
        // If the request doesn't match, return a 404 page.
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // Read the requested file's contents.
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // Form the HTTP response with headers and body.
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );

    // Write the response back to the client.
    stream.write_all(response.as_bytes()).unwrap();
}
