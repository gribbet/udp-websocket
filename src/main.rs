use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};
use std::ops::Sub;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(index = 1, default_value = "0.0.0.0:8080")]
    websocket_address: String,

    #[clap(index = 2, default_value = "0.0.0.0:14550")]
    udp_address: String,
}

fn main() {

    let Args { websocket_address, udp_address } = Args::parse();

    let (handler, listener) = node::split::<()>();

    handler
        .network()
        .listen(Transport::Udp, udp_address.clone())
        .unwrap();
    handler
        .network()
        .listen(Transport::Ws, websocket_address.clone())
        .unwrap();

    println!("Listening on ws://{} and UDP {}", websocket_address, udp_address);

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
