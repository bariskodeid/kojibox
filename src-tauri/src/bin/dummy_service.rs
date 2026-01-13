use std::env;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn main() {
    let port = env::args()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let addr = ("127.0.0.1", port);
    let listener = TcpListener::bind(addr).expect("bind failed");
    listener
        .set_nonblocking(true)
        .expect("set nonblocking failed");

    loop {
        let _ = listener.accept();
        thread::sleep(Duration::from_millis(200));
    }
}
