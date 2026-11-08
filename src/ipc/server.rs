//! The command port: localhost TCP, one JSON line in, one JSON line out, token-checked.
//! (The token keeps web pages and other users from poking the host.)

use crate::{ev::{Ev, Proxy}, protocol::Envelope};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;

pub fn spawn(listener: TcpListener, token: String, proxy: Proxy) {
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (token, proxy) = (token.clone(), proxy.clone());
            std::thread::spawn(move || handle(stream, &token, proxy));
        }
    });
}

fn handle(stream: TcpStream, token: &str, proxy: Proxy) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut line = String::new();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    if reader.by_ref().take(8_000_000).read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    let reply = match serde_json::from_str::<Envelope>(&line) {
        Err(e) => format!("{{\"ok\":false,\"err\":{}}}", serde_json::to_string(&e.to_string()).unwrap()),
        Ok(env) if env.token != token => "{\"ok\":false,\"err\":\"bad token\"}".to_string(),
        Ok(env) => {
            let (tx, rx) = mpsc::channel();
            if proxy.send_event(Ev::Cmd(env.req, Some(tx))).is_err() {
                "{\"ok\":false,\"err\":\"host closing\"}".to_string()
            } else {
                rx.recv_timeout(Duration::from_secs(5)).unwrap_or_else(|_| "{\"ok\":false,\"err\":\"timeout\"}".to_string())
            }
        }
    };
    let mut s = stream;
    let _ = writeln!(s, "{}", reply);
}
