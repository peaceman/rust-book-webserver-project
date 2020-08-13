use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs;
use std::thread;
use std::time::Duration;

use hello::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:2233").unwrap();
    let pool = ThreadPool::new(4).unwrap();

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = match buffer {
        buffer if buffer.starts_with(get) => ("HTTP/1.1 200 OK", "hello.html"),
        buffer if buffer.starts_with(sleep) => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        },
        _ => ("HTTP/1.1 404 Not Found", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line, contents.len(), contents
    );

    stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
}
