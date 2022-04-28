use anyhow::{Context, Result};
use bus_queue::Subscriber;
use log::{debug, error, info};
use matrix_sdk::{
    room::Room,
    ruma::events::{
        room::message::{MessageEventContent, MessageType},
        SyncMessageEvent,
    },
    ruma::UserId,
    Client, SyncSettings,
};
use ruma_identifiers::DeviceId;
use yarrdata::{Event, Message, ProviderQueue, SyncSubscriber};

pub struct MatrixClient {
    session: matrix_sdk::Session,
    target_room: String,
    ready: bool,
    queue: ProviderQueue,
}

impl MatrixClient {
    pub fn new(matrix_cfg: &yarrcfg::Matrix) -> Result<Self> {
        let user = UserId::try_from(matrix_cfg.user_id.clone())?;
        let session = matrix_sdk::Session {
            access_token: matrix_cfg.access_token.0.clone(),
            user_id: user,
            device_id: DeviceId::new(),
        };

        let target_room = matrix_cfg.room_id.clone();
        Ok(Self {
            session,
            target_room,
            ready: false,
            queue: ProviderQueue::new("matrix".to_owned()),
        })
    }
    pub fn subscribe(&self) -> Subscriber<Event> {
        self.queue.subscribe()
    }
    pub fn subscribe_sync(&self) -> SyncSubscriber {
        self.queue.subscribe_sync()
    }
    pub async fn run(&mut self) -> Result<()> {
        self.ready = false;
        let client = Client::new_from_user_id(self.session.user_id.clone()).await?;
        debug!("authenticating as {:?}", &self.session.user_id);
        client.restore_login(self.session.clone()).await?;
        info!("waiting for messages");
        let (tx, rx) = flume::unbounded::<(SyncMessageEvent<MessageEventContent>, Room)>();
        let jh = tokio::task::spawn(async move {
            client
                .register_event_handler(
                    move |ev: SyncMessageEvent<MessageEventContent>, room: Room| {
                        tx.send((ev, room)).unwrap();
                        async {}
                    },
                )
                .await;
            // Syncing is important to synchronize the client state with the server.
            // This method will never return.
            client.sync(SyncSettings::default()).await;
        });
        loop {
            match rx.recv_async().await {
                Ok((e, r)) => {
                    if let Err(err) = self.process_message(e, r) {
                        error!("error processing message: {:?}", err);
                    }
                }
                Err(e) => {
                    error!("error receiving messages from matrix client channel (might indicate connection closed): {:?}", e);
                    break;
                }
            }
        }
        self.ready = false;
        jh.await?;

        Ok(())
    }

    fn process_message(
        &mut self,
        ev: SyncMessageEvent<MessageEventContent>,
        room: Room,
    ) -> Result<()> {
        if !self.ready {
            self.ready = true;
            info!("receiving messages");
        }
        let room_id = room.room_id().as_str();
        if self.target_room != room_id {
            debug!("Ignored message from room ID {:?}", room_id);
        } else {
            debug!("Room {:?} >> Received a message {:?}", room.name(), ev);
            let msgid = ev.event_id.to_string();
            let username = ev.sender.localpart().to_owned();
            let timestamp = format!("{}", ev.origin_server_ts.as_secs());
            if let MessageType::Text(msg) = ev.content.msgtype {
                self.queue
                    .publish(Event::Message(Message {
                        provider_name: self.queue.provider_name.clone(),
                        message: msg.body.clone(),
                        room: room.name().unwrap_or_default(),
                        username,
                        msgid,
                        timestamp,
                    }))
                    .with_context(|| {
                        format!("trying to publish to the queue the message {:?}", msg.body)
                    })?;
            }
        }
        Ok(())
    }
}
