use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const CRLF: &str = "\r\n";
const VERSION: &str = "1.1";

fn get_response(body: &str, code: u16, message: &str) -> String {
    let status = format!("HTTP/{VERSION} {code} {message}");
    let headers = HashMap::from([
        ("Content-Length", body.len().to_string())
    ]).iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join(CRLF);
    return format!("{status}{CRLF}{headers}{CRLF}{CRLF}{body}");
}

fn on_stream(mut stream: TcpStream) {
    let reader = BufReader::new(&stream);
    let line = reader.lines().next().unwrap().unwrap();
    if line == format!("GET / HTTP/{VERSION}") {
        let body = "<html><body>Rust book: chapter 20</body></html>";
        let response = get_response(body, 200, "OK");
        println!("Response: {response}");
        stream.write_all(response.as_bytes()).unwrap();
    } else if line == format!("GET /quit HTTP/{VERSION}") {
        let body = "<html><body>bye</body></html>";
        let response = get_response(body, 200, "OK");
        println!("Response: {response}");
        stream.write_all(response.as_bytes()).unwrap();
        std::process::exit(0);
    } else {
        let body = "<html><body>Sorry, I don't know what you're asking for.</body></html>";
        let response = get_response(body, 404, "NOT FOUND");
        println!("Response: {response}");
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn main() {
    let ipv4 = "127.0.0.1";
    let port = 8080;
    let listener = TcpListener::bind(format!("{ipv4}:{port}")).unwrap();
    for it in listener.incoming() {
        on_stream(it.unwrap());
    }
}
