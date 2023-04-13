use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn on_stream(index: usize, mut stream: TcpStream) {
    // Method Request-URI HTTP-Version CRLF
    // headers CRLF
    // message-body
    let request: Vec<_> = BufReader::new(&stream)
        .lines()
        .map(|it| it.unwrap())
        .take_while(|it| !it.is_empty())
        .collect();
    println!("Request({index}): {request:#?}");

    // HTTP-Version Status-Code Reason-Phrase CRLF
    // headers CRLF
    // message-body
    let version = "1.1";
    let code = 200;
    let message = "OK";
    let crlf = "\r\n";
    let status = format!("HTTP/{version} {code} {message}");
    let body = "<html><body>Rust book: chapter 20</body></html>";
    let headers = HashMap::from([
        (String::from("Content-Length"), format!("{}", body.len()))
    ]);
    let headers = headers.iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<String>>()
        .join(crlf);
    let response = format!("{status}{crlf}{headers}{crlf}{crlf}{body}");
    println!("Response: {response}");
    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let ipv4 = "127.0.0.1";
    let port = 8080;
    let listener = TcpListener::bind(format!("{ipv4}:{port}")).unwrap();
    // let address = SocketAddr::from(([127, 0, 0, 1], port));
    // let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    // let listener = TcpListener::bind(address).unwrap();
    for (index, it) in listener.incoming().enumerate() {
        on_stream(index, it.unwrap());
    }
}
