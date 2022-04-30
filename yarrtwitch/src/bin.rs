use anyhow::{Ok, Result};
use futures::prelude::*;
use std::borrow::Borrow;
use twitch_api2::helix::{self, chat::get_global_chat_badges};
use twitch_api2::TwitchClient;
use twitch_oauth2::tokens::UserToken;
use twitch_oauth2::types::AccessToken;
use yarrcfg::parse_config;
use yarrdata::Event;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    let cfg = parse_config()?;
    let twitch_cfg = cfg.twitch.expect("Twitch config is needed to run IRC");
    // -----

    let client: TwitchClient<'static, reqwest::Client> = TwitchClient::default();
    let access_token = AccessToken::new(&twitch_cfg.oauth_token.0);
    let token = UserToken::from_existing(&client, access_token, None, None).await?;
    let request = get_global_chat_badges::GetGlobalChatBadgesRequest::new();
    let response: Vec<helix::chat::BadgeSet> = client.helix.req_get(request, &token).await?.data;
    dbg!(response);

    // -----
    let mut tw = yarrtwitch::TwitchClient::new(&twitch_cfg).await?;

    let mut twitch_sub = tw.subscribe();
    let tw_future = tokio::task::spawn(async move { tw.run().await });

    while let Some(ev) = twitch_sub.next().await {
        // Upon receiving a new message...
        match ev.borrow() {
            Event::Message(m) => {
                println!(">> {:?}>> {}", std::thread::current().id(), m.message)
            }
        }
    }
    tw_future.await??;
    Ok(())
}
