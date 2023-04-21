use std::sync::{Arc, LockResult, mpsc, Mutex};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::{Add, Rem};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    let line = reader.lines().next();
    if line.is_none() {
        println!("Read line none!");
        return;
    }
    let line = line.unwrap();
    if line.is_err() {
        println!("Read line error!");
        return;
    }
    let line = line.unwrap();
    println!("Request: {line}");
    let line = line.split(' ').collect::<Vec<_>>();
    if line.len() != 3 {
        return;
    }
    if let Some(it) = line.get(0) {
        if it != &"GET" {
            println!("Wrong method!");
            return;
        }
    }
    if let Some(it) = line.get(2) {
        if it != &format!("HTTP/{VERSION}") {
            println!("Wrong protocol!");
            return;
        }
    }
    let query = line.get(1).unwrap();
    if query.is_empty() {
        println!("Wrong query!");
        return;
    }
    match *query {
        "" => {
            println!("Wrong query!");
            return;
        }
        "/" => {
            let body = "<html><body>Rust book: chapter 20</body></html>";
            let response = get_response(body, 200, "OK");
            stream.write_all(response.as_bytes()).unwrap();
            println!("Response: {response}");
        }
        "/quit" => {
            let body = "<html><body>bye</body></html>";
            let response = get_response(body, 200, "OK");
            stream.write_all(response.as_bytes()).unwrap();
            println!("Response: {response}");
            std::process::exit(0);
        }
        "/sleep" => {
            std::thread::sleep(Duration::from_secs(5));
            let body = "<html><body>ok</body></html>";
            let response = get_response(body, 200, "OK");
            stream.write_all(response.as_bytes()).unwrap();
            println!("Response: {response}");
        }
        _ => {
            let body = "<html><body>Sorry, I don't know what you're asking for.</body></html>";
            let response = get_response(body, 404, "NOT FOUND");
            stream.write_all(response.as_bytes()).unwrap();
            println!("Response: {response}");
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // let id = SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap()
        //     .as_nanos()
        //     .rem(64);
        let handle = std::thread::spawn(move || loop {
            // let guard = receiver.lock().unwrap();
            // let job = guard.recv().unwrap();
            // any temporary values used in the expression on the right hand side
            // of the equals sign are immediately dropped when the let statement ends
            // let job = receiver.lock().unwrap().recv().unwrap();
            let job = receiver.lock().unwrap().recv();
            match job {
            // match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    println!("Worker #{id} got a job; executing...");
                    job();
                }
                Err(_) => {
                    println!("Worker #{id} disconnected; shutting down.");
                    break;
                }
            }
        });
        return Worker { handle: Some(handle) }
    }
}

impl ThreadPool {
    fn new(number: usize) -> ThreadPool {
        assert!(number > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(number);
        for index in 0..number {
            let receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::clone(&receiver);
            let worker = Worker::new(index, receiver);
            workers.push(worker);
        }
        return ThreadPool { workers, sender: Some(sender) };
    }

    fn execute<F>(&self, block: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(block);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for (index, worker) in self.workers.iter_mut().enumerate() {
            println!("Shutting down worker #{index}");
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
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
