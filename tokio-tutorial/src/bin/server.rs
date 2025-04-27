use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

// Taken from Tokio Tutorial:
//
// Concurrency and parallelism are not the same thing. If you alternate between two tasks,
// then you are working on both tasks concurrently, but not in parallel. For it to qualify
// as parallel, you would need two people, one dedicated to each task.
//
// One of the advantages of using Tokio is that asynchronous code allows you to work on
// many tasks concurrently, without having to work on them in parallel using ordinary threads.
// In fact, Tokio can run many tasks concurrently on a single thread!

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("listening!");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // clone handle
        let db = db.clone();

        // NOTE: this is a future! not a closure
        println!("accepted!");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

// std::sync::mutex vs tokio::sync::mutex -> tokio's MutexGuard is <Send>
// NOTE: be wary of holding a lock across a .await section

async fn process(socket: TcpStream, db: Db) {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(&cmd.key().to_string()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented: {:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
