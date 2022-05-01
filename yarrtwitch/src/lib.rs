use anyhow::Result;
use bus_queue::Subscriber;
use futures::StreamExt;
use irc::client::prelude::*;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::time::SystemTime;
use std::vec;
use thiserror::Error;
use twitch_api2::helix;
use twitch_api2::helix::chat::get_channel_chat_badges;
use twitch_api2::helix::chat::get_channel_emotes;
use twitch_api2::helix::chat::get_emote_sets;
use twitch_api2::helix::chat::get_global_chat_badges;
use twitch_api2::helix::chat::get_global_emotes;
use twitch_api2::TwitchClient as ApiTwitchClient;
use twitch_oauth2::tokens::UserToken;
use twitch_oauth2::types::AccessToken;
use yarrcfg::Twitch;
use yarrdata::{Badge, Event, ProviderQueue, SyncSubscriber};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client is already running, cannot run twice")]
    AlreadyRunning,
    #[error("Unexpected error: {0:?}")]
    Unexpected(String),
}

#[derive(Clone)]
pub struct Emote {
    pub id: String,
    pub name: String,
    pub image: String,
}

impl Emote {
    pub fn from_global(emote: &helix::chat::GlobalEmote) -> Self {
        Self {
            id: emote.id.to_string(),
            name: emote.name.clone(),
            image: emote.images.url_2x.clone(),
        }
    }
    pub fn from_channel(emote: &helix::chat::ChannelEmote) -> Self {
        Self {
            id: emote.id.to_string(),
            name: emote.name.clone(),
            image: emote.images.url_2x.clone(),
        }
    }
    pub fn from_emote(emote: &helix::chat::get_emote_sets::Emote) -> Self {
        Self {
            id: emote.id.to_string(),
            name: emote.name.clone(),
            image: emote.images.url_2x.clone(),
        }
    }
}

pub struct TwitchClient {
    config: irc::client::data::config::Config,
    user_token: UserToken,
    extensions: Vec<Capability>,
    ready: bool,
    queue: ProviderQueue,
    badges: Vec<helix::chat::BadgeSet>,
    emotes: HashMap<String, Emote>,
}

impl TwitchClient {
    pub async fn new(twitch_cfg: &Twitch) -> Result<Self> {
        let config = Config {
            nickname: Some(twitch_cfg.username.clone()),
            server: Some(twitch_cfg.server()),
            port: twitch_cfg.port()?,
            channels: twitch_cfg.channels.clone(),
            password: Some(format!("oauth:{}", &twitch_cfg.oauth_token.0)),
            ..Config::default()
        };
        let access_token = AccessToken::new(&twitch_cfg.oauth_token.0);
        let client: ApiTwitchClient<'static, reqwest::Client> = ApiTwitchClient::default();
        let user_token =
            UserToken::from_existing(&client, access_token.clone(), None, None).await?;

        Ok(Self {
            config,
            user_token,
            extensions: vec![Capability::Custom(":twitch.tv/tags")],
            ready: false,
            queue: ProviderQueue::new("twitch".to_owned()),
            badges: vec![],
            emotes: HashMap::new(),
        })
    }
    pub async fn get_global_emotes(&self) -> Result<Vec<helix::chat::GlobalEmote>> {
        let client: ApiTwitchClient<'static, reqwest::Client> = ApiTwitchClient::default();
        let request = get_global_emotes::GetGlobalEmotesRequest::default();
        let global_emotes: Vec<helix::chat::GlobalEmote> =
            client.helix.req_get(request, &self.user_token).await?.data;
        debug!("Global Emotes: {:?}", global_emotes);
        Ok(global_emotes)
    }
    pub async fn get_channel_emotes(&self) -> Result<Vec<helix::chat::ChannelEmote>> {
        let client: ApiTwitchClient<'static, reqwest::Client> = ApiTwitchClient::default();
        let request = get_channel_emotes::GetChannelEmotesRequest::builder()
            .broadcaster_id(self.user_token.user_id.to_string())
            .build();
        let channel_emotes: Vec<helix::chat::ChannelEmote> =
            client.helix.req_get(request, &self.user_token).await?.data;
        debug!("Channel Emotes: {:?}", channel_emotes);
        Ok(channel_emotes)
    }
    pub async fn load_emotes(&mut self) {
        let ch_emotes = match self.get_channel_emotes().await {
            Ok(x) => x,
            Err(e) => {
                error!("load_emotes: error loading channel emotes: {:?}", e);
                vec![]
            }
        };
        let gl_emotes = match self.get_global_emotes().await {
            Ok(x) => x,
            Err(e) => {
                error!("load_emotes: error loading global emotes: {:?}", e);
                vec![]
            }
        };
        for emote in ch_emotes {
            let emote = Emote::from_channel(&emote);
            self.emotes.insert(emote.id.clone(), emote);
        }
        for emote in gl_emotes {
            let emote = Emote::from_global(&emote);
            self.emotes.insert(emote.id.clone(), emote);
        }
    }
    // TODO: Not used because it doesn't return IDs for all valid emotes :-(
    fn _get_emote(&mut self, id: &str) -> Result<Emote> {
        if let Some(emote) = self.emotes.get(id) {
            return Ok(emote.to_owned());
        }
        let client: ApiTwitchClient<'static, reqwest::Client> = ApiTwitchClient::default();

        // TODO: This is horrible because we block what originally is an async block.
        use std::thread;
        // TODO: This doesn't return emotes for all valid IDs.
        let the_id = id.to_owned();
        // let the_id = "301590448".to_owned();

        let token = self.user_token.clone();
        let handle = thread::spawn(move || -> Result<Vec<get_emote_sets::Emote>> {
            use tokio::runtime::Runtime;

            // Create the runtime
            let rt = Runtime::new().unwrap();
            let request = get_emote_sets::GetEmoteSetsRequest::builder()
                .emote_set_id(vec![the_id.clone().into()])
                .build();

            debug!("get_emote({:?}) req: {:?}", the_id, &request);
            let response: Vec<helix::chat::get_emote_sets::Emote> = rt
                .block_on(async { client.helix.req_get(request, &token).await })?
                .data;
            debug!("get_emote({:?}) => {:?}", the_id, &response);
            Ok(response)
        });

        let response = handle
            .join()
            .map_err(|e| Error::Unexpected(format!("{:?}", e)))??;

        let emote = response
            .first()
            .ok_or_else(|| Error::Unexpected("no emote returned by twitch".to_owned()))?;
        let emote = Emote::from_emote(emote);
        self.emotes.insert(emote.id.clone(), emote.clone());
        Ok(emote)
    }
    fn emotes_from_str(&mut self, textemotes: &str, msg: &str) -> Vec<yarrdata::Emote> {
        // .. Tag("emotes", Some("864205:17-24/444572:0-7/724216:9-15"))
        let mut emotes: Vec<yarrdata::Emote> = vec![];
        for nb in textemotes.split('/') {
            if nb.is_empty() {
                continue;
            }
            let mut nbs = nb.split(':');
            let id = nbs.next().unwrap_or_default().to_string();
            let rangetxt = nbs.next().unwrap_or_default().to_string();
            let mut range = rangetxt.split('-');
            let from: usize = range.next().unwrap_or_default().parse().unwrap_or_default();
            let to: usize = range.next().unwrap_or_default().parse().unwrap_or_default();
            let name = msg[from..to + 1].to_owned();
            let url = format!(
                "https://static-cdn.jtvnw.net/emoticons/v2/{}/static/light/2.0",
                id
            );
            let e = yarrdata::Emote {
                id,
                from,
                to,
                name,
                url,
            };
            emotes.push(e);
            // TODO: Twitch doesn't return emotes for all valid IDs. Manually getting the URL works better :-(
            // match self.get_emote(&id) {
            //     Ok(dbemote) => {
            //         let e = yarrdata::Emote {
            //             id,
            //             from,
            //             to,
            //             name: dbemote.name.to_owned(),
            //             url: dbemote.image.to_owned(),
            //         };
            //         emotes.push(e);
            //     }
            //     Err(e) => warn!("failed to find emote {:?}: {:?}", id, e),
            // }
        }
        emotes
    }

    pub async fn get_badges(&self) -> Result<Vec<helix::chat::BadgeSet>> {
        let client: ApiTwitchClient<'static, reqwest::Client> = ApiTwitchClient::default();
        let request = get_global_chat_badges::GetGlobalChatBadgesRequest::new();
        let mut response: Vec<helix::chat::BadgeSet> =
            client.helix.req_get(request, &self.user_token).await?.data;
        let request = get_channel_chat_badges::GetChannelChatBadgesRequest::builder()
            .broadcaster_id(self.user_token.user_id.to_string())
            .build();
        let mut channel_badges = client.helix.req_get(request, &self.user_token).await?.data;
        response.append(&mut channel_badges);
        debug!("Badges: {:?}", response);
        Ok(response)
    }
    fn badges_from_str(&self, textbadges: &str) -> Vec<Badge> {
        let nbadges = textbadges.split(',');
        let mut badges: Vec<Badge> = vec![];
        for nb in nbadges {
            if nb.is_empty() {
                continue;
            }
            let mut nbs = nb.split('/');
            let name = nbs.next().unwrap_or_default().to_string();
            let vid = nbs.next().unwrap_or_default().to_string();
            let mut url = String::new();
            // TODO: Iterating across all badges might be slow - do we need a hashMap? (consider that for less than 100, iterate is faster)
            for tb in self.badges.iter() {
                if name == tb.set_id.as_str() {
                    for ver in tb.versions.iter() {
                        if ver.id.as_str() == vid {
                            url = ver.image_url_2x.to_owned();
                        }
                    }
                }
            }
            if !url.is_empty() {
                let b = Badge { name, vid, url };
                badges.push(b);
            } else {
                warn!("unable to find badge for {:?}", nb);
            }
        }
        badges
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
        self.ready = false;
        if self.badges.is_empty() {
            let badges = self.get_badges().await;
            match badges {
                Ok(badges) => self.badges = badges,
                Err(e) => error!("trying to download badges: {:?}", e),
            }
        }
        if self.emotes.is_empty() {
            self.load_emotes().await;
        }
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
        // .. Tag("subscriber", Some("0"))
        let mut msgid = String::new();
        let mut timestamp: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut username = username.clone();

        let mut badges: Vec<Badge> = vec![];
        let mut emotes: Vec<yarrdata::Emote> = vec![];
        if let Some(tags) = message.tags.as_ref() {
            for tag in tags {
                if let Some(value) = &tag.1 {
                    match tag.0.as_str() {
                        "id" => msgid = value.to_owned(),
                        "tmi-sent-ts" => {
                            timestamp = value.parse().map_or(timestamp, |x: u64| x / 1000)
                        }
                        "display-name" => username = value.to_owned(),
                        "badges" => badges = self.badges_from_str(value.as_str()),
                        "emotes" => emotes = self.emotes_from_str(value.as_str(), text),
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
            badges,
            emotes,
        });
        self.queue.publish(e)?;
        Ok(())
    }
}
