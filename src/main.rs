use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use http_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// Handle a connection.
///
/// The connection will be read and the request will be parsed.
///
/// The response will be written back to the connection.
///
/// # Panics
///
/// The `handle_connection` function will panic if the request is not a GET request.
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines()
        .next()
        .unwrap()
        .unwrap();

    let (status_line, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap()),
        "GET /sleep HTTP/1.1" => {
            std::thread::sleep(std::time::Duration::from_secs(5));
            ("HTTP/1.1 200 OK", fs::read_to_string("hello.html").unwrap())
        },
        _ => ("HTTP/1.1 404 NOT FOUND", fs::read_to_string("404.html").unwrap()),
    };

    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );

    stream.write_all(response.as_bytes()).unwrap();
}
