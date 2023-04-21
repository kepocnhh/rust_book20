use std::sync::{Arc, LockResult, mpsc, Mutex};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

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
    println!("Request: {line}");
    if line == format!("GET / HTTP/{VERSION}") {
        let body = "<html><body>Rust book: chapter 20</body></html>";
        let response = get_response(body, 200, "OK");
        stream.write_all(response.as_bytes()).unwrap();
        println!("Response: {response}");
    } else if line == format!("GET /quit HTTP/{VERSION}") {
        let body = "<html><body>bye</body></html>";
        let response = get_response(body, 200, "OK");
        stream.write_all(response.as_bytes()).unwrap();
        println!("Response: {response}");
        std::process::exit(0);
    } else if line == format!("GET /sleep HTTP/{VERSION}") {
        std::thread::sleep(Duration::from_secs(5));
        let body = "<html><body>ok</body></html>";
        let response = get_response(body, 200, "OK");
        stream.write_all(response.as_bytes()).unwrap();
        println!("Response: {response}");
    } else {
        let body = "<html><body>Sorry, I don't know what you're asking for.</body></html>";
        let response = get_response(body, 404, "NOT FOUND");
        stream.write_all(response.as_bytes()).unwrap();
        println!("Response: {response}");
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    handles: Vec<std::thread::JoinHandle<()>>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(number: usize) -> ThreadPool {
        assert!(number > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut handles = Vec::with_capacity(number);
        for index in 0..number {
            let receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::clone(&receiver);
            let handle = std::thread::spawn(move || loop {
                // let guard = receiver.lock().unwrap();
                // let job = guard.recv().unwrap();
                // any temporary values used in the expression on the right hand side
                // of the equals sign are immediately dropped when the let statement ends
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker #{index} got a job; executing...");
                job();
            });
            handles.push(handle);
        }
        return ThreadPool { handles, sender };
    }

    fn execute<F>(&self, block: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(block);
        self.sender.send(job).unwrap();
    }
}

fn main() {
    let ipv4 = "127.0.0.1";
    let port = 8080;
    let listener = TcpListener::bind(format!("{ipv4}:{port}")).unwrap();
    let pool = ThreadPool::new(4);
    for it in listener.incoming() {
        pool.execute(|| on_stream(it.unwrap()));
    }
}
