// Given a list of URLs, fetch with HTTP over multiple threads.

use std::env;

use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    thread,
};

// multi_get sends an HTTP GET request.
fn multi_get(urls: &Vec<String>) -> Vec<String> {
    // Start a thread per request. Join all threads?
    let mut handles = Vec::new();

    for url in urls {
        let url_copy = String::from(url).clone();
        handles.push(thread::spawn(|| {
            let res = reqwest::blocking::get(url_copy);
            if res.is_err() {
                return format!("Error: {}", res.err().unwrap());
            }
            let res = res.unwrap().text();
            if res.is_err() {
                return format!("Error: {}", res.err().unwrap());
            }
            return res.unwrap();
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        let res = handle.join();
        if res.is_err() {
            results.push(String::from("Error joining thread"));
        }
        results.push(res.unwrap());
    }
    return results;
}

struct TestServer {
    stopped: Mutex<bool>,
    addr: Mutex<Option<SocketAddr>>,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
}
impl TestServer {
    // Start listener thread.
    fn start() -> Arc<TestServer> {
        let ts_arc: Arc<TestServer> = Arc::new(TestServer {
            stopped: Mutex::new(false),
            addr: None.into(),
            handle: Mutex::new(None),
        });

        let ts_arc_clone = Arc::clone(&ts_arc);
        let handle = thread::spawn(move || {
            // Listen on any port.
            let listener = TcpListener::bind("localhost:0").unwrap();
            let addr = listener.local_addr().unwrap();
            println!("Test server is listening on address: {:?}", addr);
            *ts_arc_clone.addr.lock().unwrap() = Some(addr);

            // Accept connections.
            loop {
                let got = listener.accept();
                let (mut stream, addr) = got.expect("failed to receive stream");
                println!("test server got connection from address: {}", addr);
                if *(ts_arc_clone.stopped.lock().unwrap()) {
                    println!("test server detected stop signal");
                    return;
                }
                // Spawn thread to handle request.
                thread::spawn(move || {
                    // Receive request.
                    let mut reader = BufReader::new(&stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).expect("should read line");
                    if line.len() == 0 {
                        // Connection may have closed.
                        return;
                    }
                    println!("test server got message: {}", line);
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    // Check for HTTP GET first line. Example: `GET / HTTP/1.1`
                    assert!(parts.len() >= 2, "expected HTTP GET request, got: {}", line);
                    assert_eq!(
                        parts[0], "GET",
                        "Test server only supports GET requests, got: {}",
                        line
                    );
                    let path = parts[1];
                    let resp_str = format!("Sample reply to: {}", path);

                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\n")
                        .expect("failed to write");
                    stream
                        .write_all(b"Date: Wed, 10 Aug 2023 15:30:00 GMT\r\n")
                        .expect("failed to write");
                    stream
                        .write_all(b"Server: Apache/2.4.41 (Unix)\r\n")
                        .expect("failed to write");
                    stream
                        .write_all(format!("Content-Length: {}\r\n", resp_str.len()).as_bytes())
                        .expect("failed to write");
                    stream
                        .write_all(b"Content-Type: text/plain; charset=utf-8\r\n")
                        .expect("failed to write");
                    stream.write(b"\r\n").expect("failed to write");
                    stream
                        .write_all(format!("{}\r\n", resp_str).as_bytes())
                        .expect("failed to write");
                });
            }
        });

        *ts_arc.handle.lock().unwrap() = Some(handle);

        return ts_arc;
    }

    // Get address. `start` must have been called prior.
    fn addr(&self) -> Option<SocketAddr> {
        let res = self.addr.lock();
        // Panic on failure to lock.
        let res = res.unwrap();
        return *res;
    }

    // Stop and join listener thread. `start` must have been called prior.
    fn stop(&self) {
        println!("Setting stopped");
        // Set `stopped`.
        *self.stopped.lock().unwrap() = true;
        println!("Connecting again");
        let _ = TcpStream::connect(self.addr().unwrap()); // Ignore result. Connect may expectedly fail if another thread connects before.
        println!("Joining thread");
        {
            self.handle
                .lock()
                .unwrap()
                .take()
                .unwrap()
                .join()
                .expect("should join server thread");
        }
    }

    /// Wait until the test server can accept connections.
    /// Returns the address of the server.
    fn await_listen(&self) -> SocketAddr {
        // Wait for address.
        let addr: SocketAddr;
        let start = std::time::SystemTime::now();
        loop {
            let diff = start.elapsed().unwrap();
            if diff > std::time::Duration::from_secs(1) {
                panic!("Loop exceeded timeout");
            }
            let maybe_addr = self.addr();
            if maybe_addr.is_some() {
                addr = maybe_addr.unwrap();
                break;
            }

            // Sleep, and try again.
            println!("No address. Sleeping and trying again");
            thread::sleep(std::time::Duration::from_millis(50));
        }

        // Wait for connect to succeed.
        let start = std::time::SystemTime::now();
        loop {
            let diff = start.elapsed().unwrap();
            if diff > std::time::Duration::from_secs(1) {
                panic!("Loop exceeded timeout");
            }
            if TcpStream::connect(addr).is_err() {
                println!("Failed to connect. Sleeping and trying again");
                thread::sleep(std::time::Duration::from_millis(50));
            }
            break;
        }

        return addr;
    }
}

#[test]
fn test_TestServer() {
    // Self-test the test server
    let ts_arc = TestServer::start();
    // Get the address of the test server.
    let addr = ts_arc.await_listen();
    let host_port = addr.to_string();

    // Test one request.
    {
        let url = format!("http://{}", host_port);
        let resp = reqwest::blocking::get(url.as_str()).expect("should send GET request");
        let resp = resp.text().expect("expected to read reply");
        assert_eq!(resp, "Sample reply to: /");
    }

    // Test two concurrent requests.
    {
        let host_port_t1 = host_port.clone();
        let t1_handle = thread::spawn(move || {
            let url = format!("http://{}/1", host_port_t1);
            let resp = reqwest::blocking::get(url.as_str()).expect("should send GET request");
            let resp = resp.text().expect("should read reply");
            assert_eq!(resp, "Sample reply to: /1");
        });
        let host_port_t2 = host_port.clone();
        let t2_handle = thread::spawn(move || {
            let url = format!("http://{}/2", host_port_t2);
            let resp = reqwest::blocking::get(url.as_str()).expect("should send GET request");
            let resp = resp.text().expect("should read reply");
            assert_eq!(resp, "Sample reply to: /2");
        });
        t1_handle.join().expect("should join thread 1");
        t2_handle.join().expect("should join thread 2");
    }

    ts_arc.stop();
}
#[test]
fn test_multi_get() {
    let ts = TestServer::start();
    let addr = ts.await_listen();
    let host_port = addr.to_string();
    let url1 = format!("http://{}/1", host_port);
    let url2 = format!("http://{}/2", host_port);
    let got = multi_get(&vec![url1, url2]);
    assert_eq!(got, vec!["Sample reply to: /1", "Sample reply to: /2",]);
    ts.stop();
}

fn main() {
    let urls: Vec<String> = std::env::args().skip(1).collect();
    println!("Sending requests ... begin");
    let responses = multi_get(&urls);
    println!("Sending requests ... end");

    for (i, url) in urls.iter().enumerate() {
        let res = &responses[i];
        let title_start = res.find("<title>");
        let title_end = res.find("</title>");
        if title_start.is_some() && title_end.is_some() {
            let title_start = title_start.unwrap();
            let title_end = title_end.unwrap();
            let title = &res[title_start + "<title>".len()..title_end];
            println!("Response to {}:\n  Title: {}", url, title);
        } else {
            println!("Response to {}:\n  Could not find <title>", url);
        }
    }
}
