use anyhow::{Ok, Result};
use futures::prelude::*;
use std::borrow::Borrow;
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
    let mut tw = yarrtwitch::TwitchClient::new(&twitch_cfg)?;

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
