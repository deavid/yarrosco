extern crate bus_queue;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use bus_queue::{bounded, Publisher, Subscriber};
use futures::executor::block_on;
use futures::{FutureExt, SinkExt, StreamExt};

#[derive(Debug, Clone)]
pub enum Event {
    Message(Message),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub provider_name: String,
    pub room: String,
    pub message: String,
    pub username: String,
    pub msgid: String,
    pub timestamp: u64,
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
