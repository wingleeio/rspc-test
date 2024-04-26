use std::sync::{Arc, Mutex};

use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

pub struct Emitter<T> {
    listeners: Mutex<HashMap<String, Vec<Sender<T>>>>,
}

impl<T: Send + Clone + 'static> Emitter<T> {
    pub fn new() -> Arc<Self> {
        Arc::new(Emitter {
            listeners: Mutex::new(HashMap::new()),
        })
    }
    pub fn add_listener(&self, event: String, sender: Sender<T>) {
        let mut listeners = self.listeners.lock().unwrap();

        listeners.entry(event).or_insert_with(Vec::new).push(sender);
    }

    pub fn remove_listener(&self, event: String, sender: &Sender<T>) {
        if let Some(senders) = self.listeners.lock().unwrap().get_mut(event.as_str()) {
            senders.retain(|s| !s.same_channel(sender));
        }
    }

    pub fn emit(&self, event: &str, data: T) {
        if let Some(senders) = self.listeners.lock().unwrap().get(event) {
            for sender in senders {
                let _ = sender.try_send(data.clone());
            }
        }
    }
}
