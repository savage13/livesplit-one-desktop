use serde::Serialize;
use simple_websockets::{Event, EventHub, Message, Responder};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;

async fn ws_event(event_hub: &EventHub, clients: &mut HashMap<u64, Responder>) {
    match event_hub.poll_async().await {
        Event::Connect(client_id, responder) => {
            println!("A client connected with id #{}", client_id);
            // add their Responder to our `clients` map:
            clients.insert(client_id, responder);
        }
        Event::Disconnect(client_id) => {
            println!("Client #{} disconnected.", client_id);
            // remove the disconnected client from the clients map:
            clients.remove(&client_id);
        }
        Event::Message(client_id, message) => {
            println!(
                "Received a message from client #{}: {:?}",
                client_id, message
            );
            // retrieve this client's `Responder`:
            let responder = clients.get(&client_id).unwrap();
            // echo the message back:
            responder.send(message);
        }
    }
}

async fn local_event(rx: &mut mpsc::UnboundedReceiver<Msg>) -> Msg {
    dbg!("local event");
    match rx.recv().await {
        None => {
            println!("receiver dropped");
            return Msg::Done;
        }
        Some(v) => {
            return v;
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
enum Msg {
    Value(serde_json::Value),
    Done,
}

async fn ws_server(mut rx: mpsc::UnboundedReceiver<Msg>) {
    // listen for WebSockets on port 8080:
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
    // map between client ids and the client's `Responder`:
    let mut clients: HashMap<u64, Responder> = HashMap::new();

    loop {
        tokio::select!(
            x = local_event(&mut rx) => {
                println!("local event {:?}", x);
                match x {
                    Msg::Done =>                      break,
                    Msg::Value(v) => {
                        for (_, responder) in &clients {
                            responder.send(Message::Text(format!("{}",v)));
                        }
                    }
                }
            },
            _ = ws_event(&event_hub, &mut clients) => {
                println!("web server event")
            },
        );
    }
}

pub struct Notifier {
    runtime: Option<tokio::runtime::Runtime>,
    tx: mpsc::UnboundedSender<Msg>,
}
impl Notifier {
    pub fn new() -> Notifier {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        let _handle = runtime.spawn(ws_server(rx));
        Self {
            runtime: Some(runtime),
            tx,
        }
    }
    pub fn close(&mut self) -> Option<()> {
        self.tx.send(Msg::Done).ok()?;
        //self.runtime.block_on(self.handle).unwrap();
        let runtime = self.runtime.take().unwrap();
        runtime.shutdown_timeout(Duration::from_millis(100));
        Some(())
    }
    pub fn send<T: Serialize>(&mut self, value: T) -> Option<()> {
        let value = serde_json::to_value(value).unwrap();
        self.tx.send(Msg::Value(value)).ok()?;
        Some(())
    }
}
