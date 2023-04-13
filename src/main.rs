use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

fn on_stream(index: usize, stream: TcpStream) {
    let request: Vec<_> = BufReader::new(stream)
        .lines()
        .map(|it| it.unwrap())
        .take_while(|it| !it.is_empty())
        .collect();
    println!("Request({index}): {request:#?}");
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
