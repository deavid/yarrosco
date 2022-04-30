extern crate bus_queue;
pub mod db;
use anyhow::Result;
use bus_queue::{bounded, Publisher, Subscriber};
use futures::executor::block_on;
use futures::{FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Message(Message),
}

impl Event {
    pub fn from_json(text: &str) -> Result<Self> {
        let m: Self = serde_json::from_str(text.trim())?;
        Ok(m)
    }
    pub fn to_json(&self) -> Result<String> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::Message(m) => m.timestamp,
        }
    }
}

fn default_provider() -> String {
    "no-provider".to_string()
}

fn default_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    // To add robustness when deserializing, we must have defaults for everything.
    #[serde(default = "default_provider")]
    pub provider_name: String,
    #[serde(default)]
    pub room: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub msgid: String,
    #[serde(default = "default_timestamp")]
    pub timestamp: u64,
    #[serde(default)]
    pub badges: Vec<Badge>,
    #[serde(default)]
    pub emotes: Vec<Emote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Badge {
    // broadcaster/1 -> name: broadcaster, vid: 1
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub vid: String,

    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Emote {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub from: usize,
    #[serde(default)]
    pub to: usize,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
}

pub struct ProviderQueue {
    pub provider_name: String,
    pub publisher: Publisher<Event>,
    pub subscriber: Subscriber<Event>,
}

impl ProviderQueue {
    const SIZE: usize = 1024;
    pub fn new(provider_name: String) -> Self {
        let (pb, sb) = bounded::<Event>(Self::SIZE);
        Self {
            provider_name,
            publisher: pb,
            subscriber: sb,
        }
    }
    pub fn close(&mut self) -> Result<()> {
        Ok(block_on(async move { self.publisher.close().await })?)
    }
    pub fn publish(&mut self, e: Event) -> Result<()> {
        Ok(block_on(async move { self.publisher.send(e).await })?)
    }
    pub fn subscribe(&self) -> Subscriber<Event> {
        self.subscriber.clone()
    }
    pub fn subscribe_sync(&self) -> SyncSubscriber {
        SyncSubscriber {
            subscriber: self.subscriber.clone(),
        }
    }
}

pub enum Status {
    Event(Arc<Event>),
    Closed,
    Error,
}

impl Status {
    pub fn new(v: Option<Option<Arc<Event>>>) -> Self {
        match v {
            None => Self::Closed,
            Some(x) => match x {
                None => Self::Error,
                Some(y) => Self::Event(y),
            },
        }
    }
}

pub struct SyncSubscriber {
    pub subscriber: Subscriber<Event>,
}
impl SyncSubscriber {
    pub fn get_next(&mut self) -> Status {
        Status::new(self.subscriber.next().now_or_never())
    }
    pub fn peek_next(&mut self) -> Status {
        let mut p = self.subscriber.clone().peekable();
        let peek = Pin::new(&mut p).peek();
        Status::new(peek.now_or_never().map(|x| x.cloned()))
    }
}
