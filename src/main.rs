#[allow(unused_imports)]
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
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

        (status_line, content_type, contents) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "", ""),
            "GET /echo/ HTTP/1.1" => ("HTTP/1.1 200 OK", "text/plain", request_line.strip_prefix("GET /echo/").unwrap().strip_suffix(" HTTP/1.1").unwrap()),
            "GET /user-agent HTTP/1.1" => {
                let user_agent = http_request
                                .iter()
                                .find(|line| line.starts_with("User-Agent: "))
                                .map(|line| line.trim_start_matches("User-Agent: "))
                                .unwrap_or("");
                ("HTTP/1.1 200 OK", "text/plain", user_agent)
            }
            _ => ("HTTP/1.1 404 Not Found", "", "")
        };
    } 

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\n\r\n{contents}");
    tcp_stream.write(response.as_bytes()).unwrap();
    tcp_stream.flush().unwrap();
}

