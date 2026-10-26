use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::Notify;

use crate::model::{Status, UsageSnapshot};
use crate::providers;
use crate::store::Store;

pub struct AppState {
    pub store: Mutex<Store>,
    pub snapshots: Mutex<HashMap<String, UsageSnapshot>>,
    pub client: Client,
    pub wake: Notify,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        let client = Client::builder()
            .user_agent(concat!("ai-usage/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(20))
            .build()
            .expect("failed to build HTTP client");
        Self {
            store: Mutex::new(store),
            snapshots: Mutex::new(HashMap::new()),
            client,
            wake: Notify::new(),
        }
    }

    pub fn ordered_snapshots(&self) -> Vec<UsageSnapshot> {
        let store = self.store.lock().unwrap();
        let snapshots = self.snapshots.lock().unwrap();
        store
            .accounts
            .iter()
            .map(|a| {
                snapshots
                    .get(&a.id)
                    .cloned()
                    .unwrap_or_else(|| providers::placeholder(a, Status::Loading))
            })
            .collect()
    }
}
