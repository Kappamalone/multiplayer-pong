use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(msg) = rx.recv().await {
            match msg {
                Command::Get { key, resp } => {
                    let _ = resp.send(client.get(&key).await);
                }
                Command::Set { key, val, resp } => {
                    let _ = resp.send(client.set(&key, val.into()).await);
                }
            };
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        tx.send(Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await.unwrap();
        println!("response: {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        tx2.send(Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await.unwrap();
        println!("response: {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

    Ok(())
}
