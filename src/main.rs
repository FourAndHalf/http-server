#[allow(unused_imports)]
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration
};
use http_server::ThreadPool;

enum Route {
    Root,
    Echo,
    UserAgent,
    File,
    NotFound
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(6);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}

fn handle_connection(mut tcp_stream: TcpStream) {
    let buf_reader = BufReader::new(&tcp_stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result|result.unwrap())
        .take_while(|line|!line.is_empty())
        .collect();

    let mut status_line = "HTTP/1.1 404 Not Found";
    let mut content_type = "";
    let mut contents = "";

    if !http_request.is_empty() {
        let request_line: &str = &http_request[0];
        let route = get_route_type(request_line);

        (status_line, content_type, contents) = match route {
            Route::Root => ("HTTP/1.1 200 OK", "", ""),
            Route::Echo => {
                let text = request_line
                    .strip_prefix("GET /echo/")
                    .and_then(|s| s.strip_suffix(" HTTP/1.1"))
                    .unwrap();

                ("HTTP/1.1 200 OK", "text/plain", text)
            },
            Route::UserAgent => {
                let user_agent = http_request
                                .iter()
                                .find(|line| line.starts_with("User-Agent: "))
                                .map(|line| line.trim_start_matches("User-Agent: "))
                                .unwrap_or("");
                ("HTTP/1.1 200 OK", "text/plain", user_agent)
            },
            Route::File => {
                let file_name: &str = request_line
                    .strip_prefix("GET /files/")
                    .and_then(|s| s.strip_suffix(" HTTP/1.1"))
                    .unwrap();
                let full_path: String = format!("{base_path}{file_name}");
                let content_full_path: &str = &full_path;
                let path = Path::new(&full_path);
                if path.exists() && path.is_file() {
                    ("HTTP/1.1 200 OK", "application/octet", content_full_path)
                } else {
                    ("HTTP/1.1 404 Not Found", "", "")
                }
            },
            Route::NotFound => ("HTTP/1.1 404 Not Found", "", "")
        };
    } 

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\n\r\n{contents}");
    tcp_stream.write(response.as_bytes()).unwrap();
    tcp_stream.flush().unwrap();
}

fn get_route_type(request_line: &str) -> Route {
    if request_line == "GET / HTTP/1.1" {
        Route::Root
    } else if request_line.starts_with("GET /echo/") && request_line.ends_with(" HTTP/1.1") {
        Route::Echo
    } else if request_line == "GET /user-agent HTTP/1.1" {
        Route::UserAgent
    } else if request_line.starts_with("GET /files/") && request_line.ends_with(" HTTP/1.1") {
        Route::File
    } else {
        Route::NotFound
    }
}
