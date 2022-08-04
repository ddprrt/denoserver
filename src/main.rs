use std::thread;

use tokio::sync::mpsc;

mod platform;
mod server;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(async move {
        server::Server::start(tx).await;
    });

    while let Some((path, sender)) = rx.recv().await {
        thread::spawn(|| {
            let worker = platform::Worker::default();
            worker.execute(path);
            sender.send("💪".to_string()).unwrap();
        });
    }
}
