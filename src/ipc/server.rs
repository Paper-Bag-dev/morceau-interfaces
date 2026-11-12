//! The command port: one JSON line in, one JSON line out, token-checked, over the transport (transport.rs: a local socket only this user
//! can open by default, or localhost TCP with MORC_TRANSPORT=tcp). The token keeps other programs from poking the host.

use crate::{ev::{Ev, Proxy}, protocol::Envelope, transport::{Conn, Listener}};
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::mpsc;
use std::time::Duration;

pub fn spawn(listener: Listener, token: String, proxy: Proxy) {
    listener.serve(move |conn| handle(conn, &token, proxy.clone()));
}

fn handle(mut conn: Box<dyn Conn>, token: &str, proxy: Proxy) {
    let mut line = String::new();
    {
        let mut reader = BufReader::new(&mut conn);
        if reader.by_ref().take(8_000_000).read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
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
    let _ = writeln!(conn, "{}", reply);
    let _ = conn.flush();
}
