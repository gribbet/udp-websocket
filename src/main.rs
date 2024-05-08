use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};
use std::ops::Sub;

fn main() {
    let websocket_address = "0.0.0.0:8080";
    let udp_address = "0.0.0.0:14550";

    let (handler, listener) = node::split::<()>();

    handler
        .network()
        .listen(Transport::Udp, udp_address)
        .unwrap();
    handler
        .network()
        .listen(Transport::Ws, websocket_address)
        .unwrap();

    println!("Listening on {} and {}", websocket_address, udp_address);

    let endpoints = Endpoints::new();

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => (),
        NetEvent::Accepted(endpoint, _) => {
            println!("Client connected");
            endpoints.add(endpoint);
        }
        NetEvent::Message(endpoint, data) => {
            endpoints.add(endpoint);
            for receiver in endpoints.list() {
                if endpoint.resource_id().adapter_id() != receiver.resource_id().adapter_id() {
                    handler.network().send(receiver, data);
                }
            }
        }
        NetEvent::Disconnected(endpoint) => {
            println!("Client disconnected");
            endpoints.remove(endpoint);
        }
    });
}

struct Endpoints {
    endpoints: Arc<Mutex<HashMap<Endpoint, Instant>>>,
}

impl Endpoints {
    fn new() -> Endpoints {
        Endpoints {
            endpoints: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add(&self, endpoint: Endpoint) {
        self.endpoints
            .clone()
            .lock()
            .unwrap()
            .insert(endpoint, Instant::now());
    }

    fn remove(&self, endpoint: Endpoint) {
        self.endpoints.clone().lock().unwrap().remove(&endpoint);
    }

    fn list(&self) -> Vec<Endpoint> {
        self.endpoints
            .clone()
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(_, instant)| instant.ge(&Instant::now().sub(Duration::from_secs(10))))
            .map(|(endpoint, _)| endpoint)
            .collect()
    }
}
