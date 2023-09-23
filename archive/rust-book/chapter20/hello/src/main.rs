use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use hello::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let first_line = buf_reader.lines().next().unwrap().unwrap();
    println!("first_line={}", first_line);

    let status_code;
    let file_path;

    if first_line == "GET / HTTP/1.1" {
        status_code = 200;
        file_path = "hello.html"
    } else if first_line == "GET /sleep HTTP/1.1" {
        status_code = 200;
        thread::sleep(Duration::from_secs(5));
        file_path = "hello.html"
    } else {
        status_code = 404;
        file_path = "404.html";
    }

    match &first_line[..] {
        "GET / HTTP/1.1" => {
            println!("Returning root");
        }
        "GET /sleep HTTP/1.1" => {
            println!("Sleeping and returning root");
        }
        _ => {
            println!("Returning 404");
        }
    };

    stream
        .write_all(format!("HTTP/1.1 {} OK\r\n", status_code).as_bytes())
        .expect("failed to write");

    let contents = fs::read_to_string(file_path).unwrap();
    let length = contents.len();
    stream
        .write_all(format!("Content-Length: {}\r\n\r\n", length).as_bytes())
        .expect("failed to write");
    stream
        .write_all(contents.as_bytes())
        .expect("failed to write");
    stream.write(b"\r\n").expect("failed to write");
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::build(4).unwrap();
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        pool.execute(|| {
            println!(
                "Connection established from: {}",
                stream.peer_addr().unwrap()
            );
            handle_connection(stream);
        });
    }
}
