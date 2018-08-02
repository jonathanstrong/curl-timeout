extern crate curl;
extern crate thhp;

use std::thread;
use std::net::{SocketAddr, ToSocketAddrs, TcpListener};
use std::time::Duration;
use std::process::exit;
use std::io::{Read, Write, stdout};
use std::str::from_utf8;
use curl::easy::Easy;

const HTTP_204: &[u8] = b"HTTP/1.1 204 No Content\r\n\r\n";

fn lackadaisical_server<A: ToSocketAddrs>(addr: A) -> thread::JoinHandle<()> {
    let addr: SocketAddr = addr.to_socket_addrs().unwrap().next().unwrap();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        let socket = TcpListener::bind(addr).expect("TcpListener::bind");
        println!("[server] listening...");
        'a: for stream in socket.incoming() {
            if let Ok(mut stream) = stream {
                let mut i = 0;
                'b: loop {
                    if let Ok(n) = stream.read(&mut buf[i..]) { i += n }

                    let mut headers: Vec<thhp::HeaderField> = Vec::with_capacity(16);

                    match thhp::Request::parse(&buf[..i], &mut headers) {
                        Ok(thhp::Status::Complete(_)) => {
                            println!("[server] new request\n\n{}", from_utf8(&buf[..i]).expect("from_utf8"));
                            break 'b
                        }

                        Ok(thhp::Status::Incomplete) => {
                            println!("still incomplete");
                        }

                        Err(e) => {
                            panic!("error trying to parse request: {:?}", e);
                        }
                    }
                }
                println!("[server] delaying response 3 seconds...");
                thread::sleep(Duration::from_secs(3));
                stream.write_all(HTTP_204).expect("write_all");
                println!("[server] finished response");
            }
        }
    })
}

fn main() {
    let _server = lackadaisical_server("127.0.0.1:4115");
    thread::sleep(Duration::from_millis(100));
    let mut handle = Easy::new();
    handle.timeout(Duration::from_secs(2)).expect("timeout");
    handle.url("127.0.0.1:4115").expect("url");
    handle.write_function(|data| {
        Ok(stdout().write(data).expect("stdout().write(data)"))
    }).expect("handle.write_function");
    match handle.perform() {
        Ok(_) => {
            println!("[client] handle.perform() returned Ok(_)");
            exit(1)
        }

        Err(e) => {
            println!("[client] handle.perform() returned Err({})", e);
            exit(0)
        }
    }
}
