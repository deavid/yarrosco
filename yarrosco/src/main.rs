use anyhow::Result;
use futures::StreamExt;
use log::LevelFilter;
use std::{borrow::Borrow, sync::Arc};
use tokio::task;
use yarrdata::Event;
use yarrmatrix::MatrixClient;
use yarrtwitch::TwitchClient;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::builder()
        .filter(Some("sled"), LevelFilter::Info)
        .filter(Some("reqwest"), LevelFilter::Info)
        .init();

    let cfg = yarrcfg::parse_config()?;
    // Create yarrtwitch (spawn a thread?)
    // TODO: remove the expect, twitch is not mandatory really.
    let twitch_cfg = cfg.twitch.expect("Twitch config is needed");

    let mut tw = TwitchClient::new(&twitch_cfg)?;
    // Subscribe to twitch
    let mut twitch_sub = tw.subscribe();
    let tw_future = task::spawn(async move { tw.run().await });

    // Create and connect to matrix
    let matrix_cfg = cfg.matrix.expect("Matrix config is needed");

    let mut mx = MatrixClient::new(&matrix_cfg)?;
    // Subscribe to matrix
    let mut matrix_sub = mx.subscribe();
    let mx_future = task::spawn(async move { mx.run().await });

    // Upon receiving a new matrix message...
    loop {
        tokio::select! {
            Some(v) = twitch_sub.next() => process_message("twitch", v),
            Some(v) = matrix_sub.next() => process_message("matrix", v),
            else => break,
        }
    }

    tw_future.await??;
    mx_future.await??;
    Ok(())
}

fn process_message(provider: &str, ev: Arc<Event>) {
    match ev.borrow() {
        Event::Message(m) => {
            println!("#{}::{}> {}", provider, m.username, m.message)
        }
    }
}
