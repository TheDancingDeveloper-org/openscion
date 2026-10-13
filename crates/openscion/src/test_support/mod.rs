//! Servers for the client's unit tests: a real `openscion-server`, and a
//! canned one that answers every request with a fixed body.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use openscion_server::{Server, ServerConfig};

/// Starts a real server on a free loopback port and returns its base URL.
///
/// The server runs on its own runtime thread until the test process exits.
pub(crate) fn real_server() -> String {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let server = Server::bind(&ServerConfig::new("127.0.0.1:0".parse().unwrap()))
                .await
                .unwrap();
            sender.send(server.local_addr()).unwrap();
            server.serve(std::future::pending()).await.unwrap();
        });
    });
    format!("http://{}", receiver.recv().unwrap())
}

/// Starts a server that answers every request with `status` and `body`, and
/// returns its base URL.
pub(crate) fn canned_server(status: u16, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap() > 2 {
                line.clear();
            }
            write!(
                stream,
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    format!("http://{addr}")
}

/// A base URL nothing is listening on.
pub(crate) fn unreachable_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    format!("http://{addr}")
}
