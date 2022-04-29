use std::time::SystemTime;

use anyhow::Result;
use bus_queue::Subscriber;
use futures::StreamExt;
use irc::client::prelude::*;
use log::{debug, error, info, warn};
use thiserror::Error;
use yarrcfg::Twitch;
use yarrdata::{Event, ProviderQueue, SyncSubscriber};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client is already running, cannot run twice")]
    AlreadyRunning,
}

pub struct TwitchClient {
    config: irc::client::data::config::Config,
    extensions: Vec<Capability>,
    // running: Mutex<()>,
    ready: bool,
    queue: ProviderQueue,
}

impl TwitchClient {
    pub fn new(twitch_cfg: &Twitch) -> Result<Self> {
        let config = Config {
            nickname: Some(twitch_cfg.username.clone()),
            server: Some(twitch_cfg.server()),
            port: twitch_cfg.port()?,
            channels: twitch_cfg.channels.clone(),
            password: Some(format!("oauth:{}", &twitch_cfg.oauth_token.0)),
            ..Config::default()
        };
        Ok(Self {
            config,
            extensions: vec![Capability::Custom(":twitch.tv/tags")],
            ready: false,
            // running: Mutex::default(),
            queue: ProviderQueue::new("twitch".to_owned()),
        })
    }
    pub fn subscribe(&self) -> Subscriber<Event> {
        self.queue.subscribe()
    }
    pub fn subscribe_sync(&self) -> SyncSubscriber {
        self.queue.subscribe_sync()
    }
    pub async fn run(&mut self) -> Result<()> {
        for _ in 0..16 {
            if let Err(e) = self.run_once().await {
                error!("On IRC connection: {:?}", e);
            }
        }
        warn!("end of connection retries");
        Ok(())
    }
    async fn run_once(&mut self) -> Result<()> {
        // let _running = self.running.try_lock().map_err(|_| Error::AlreadyRunning)?;
        self.ready = false;
        let mut client = Client::from_config(self.config.clone()).await?;
        client.identify()?;
        client.send_cap_req(&self.extensions)?;
        let mut stream = client.stream()?;
        // *** No question mark operator from here ---
        let mut err_count = 0;
        while let Some(resmessage) = stream.next().await {
            match resmessage {
                Ok(message) => {
                    if let Err(e) = self.process_stream(&message) {
                        error!("error processing message {:?}: {:?}", &message, e);
                    } else {
                        err_count = 0;
                    }
                }
                Err(e) => {
                    error!("error while processing IRC stream: {:?}", e);
                    err_count += 1;
                    if err_count > 10 {
                        error!("too many consecutive errors, giving up");
                        break;
                    }
                }
            }
        }
        // *** to here --- (? operator)
        // Sometimes, specially when connecting, it kills the connection before auth.
        warn!("IRC Connection ended (ready={:?})", self.ready);
        self.ready = false;
        Ok(())
    }

    fn process_stream(&mut self, message: &Message) -> Result<()> {
        match &message.command {
            Command::PING(_, _) | Command::PONG(_, _) => {}
            Command::NOTICE(tgt, msg) => {
                info!("NOTICE[{}]: {}", tgt, msg);
            }
            Command::Response(r, data) => match r.is_error() {
                true => error!("E[{:?}] < {:?}", r, data),
                false => {
                    match r {
                        Response::RPL_MOTD | Response::RPL_MOTDSTART => {}
                        Response::RPL_ENDOFMOTD => {
                            info!("IRC Connection successful.");
                            self.ready = true;
                        }
                        _ => debug!("< [{:?}] {:?}", r, data),
                    };
                }
            },
            Command::PRIVMSG(tgt, msg) => self.process_msg(tgt, msg, message)?,
            c => debug!(": {:?}", c),
        }
        Ok(())
    }
    fn process_msg(&mut self, target: &str, text: &str, message: &Message) -> Result<()> {
        use yarrdata::Message;
        let username = match message.prefix.as_ref().unwrap() {
            Prefix::ServerName(sn) => sn,
            Prefix::Nickname(nick, _user, _host) => nick,
        };
        debug!(
            "P.Msg[{}]: {} (pfix: {:?} tags: {:?}) thread: {:?}",
            target,
            text,
            message.prefix,
            message.tags,
            std::thread::current().id(),
        );
        // TODO: Tag("id", Some("65a27446-8c5c-4a5a-9f18-c759824d1e69"))
        // TODO: Tag("tmi-sent-ts", Some("1651155343269"))
        // .. Tag("display-name", Some("UserName"))
        // .. Tag("subscriber", Some("0"))
        // .. Tag("badge-info", Some("")), Tag("badges", Some("broadcaster/1"))
        // .. Tag("emotes", Some("")) <--- samples?
        let mut msgid = String::new();
        let mut timestamp: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut username = username.clone();

        if let Some(tags) = message.tags.as_ref() {
            for tag in tags {
                if let Some(value) = &tag.1 {
                    match tag.0.as_str() {
                        "id" => msgid = value.to_owned(),
                        "tmi-sent-ts" => {
                            timestamp = value.parse().map_or(timestamp, |x: u64| x / 1000)
                        }
                        "display-name" => username = value.to_owned(),
                        _ => {}
                    }
                }
            }
        }
        let e = Event::Message(Message {
            provider_name: self.queue.provider_name.clone(),
            room: target.to_owned(),
            message: text.to_owned(),
            username,
            msgid,
            timestamp,
        });
        self.queue.publish(e)?;
        Ok(())
    }
}
