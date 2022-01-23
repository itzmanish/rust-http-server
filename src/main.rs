use rust_http_server::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("localhost:3000").unwrap();
    let pool = ThreadPool::new(4).unwrap();

    println!("TCP server is running at localhost:3000");
    for conn in listener.incoming() {
        let conn = conn.unwrap();
        println!("connection established!");
        pool.execute(|| {
            handle_connection(conn);
        })
    }
}

fn handle_connection(mut conn: TcpStream) {
    let mut buffer = [0; 1024];

    conn.read(&mut buffer).unwrap();

    let get_req = b"GET / HTTP/1.1\r\n";
    let sleep_req = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get_req) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep_req) {
        std::thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let html_data = fs::read_to_string(filename).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        html_data.len(),
        html_data
    );
    conn.write(response.as_bytes()).unwrap();
    conn.flush().unwrap();
}
