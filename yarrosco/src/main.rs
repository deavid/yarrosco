use anyhow::Result;
use futures::StreamExt;
use log::LevelFilter;
use log::{error, info};
use std::{borrow::Borrow, sync::Arc};
use tokio::task;
use yarrdata::db::{self, MessageIgnored};
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

    let mut log = db::Log::new(100, cfg.logfile, cfg.checkpointfile);
    if let Err(e) = log.load().await {
        error!("couldn't load the database: {:?}", e);
    }
    for (_, ce) in log.data.iter() {
        process_message(&ce.event);
    }
    // TODO: Implement a yarrosco-secondary to have as a background + backup (name: yarrly? yarrdy? female-parrot)

    // Upon receiving a new matrix message...
    loop {
        tokio::select! {
            Some(v) = twitch_sub.next() => process_message_log(&mut log, v).await,
            Some(v) = matrix_sub.next() => process_message_log(&mut log, v).await,
            else => break,
        }
    }

    tw_future.await??;
    mx_future.await??;
    Ok(())
}

async fn process_message_log(logger: &mut db::Log, ev: Arc<Event>) {
    let event: &Event = ev.borrow();

    match logger.push(event.clone()).await {
        Ok(MessageIgnored::None) => process_message(event),
        Ok(reason) => info!("ignored message {:?}: {:?}", reason, event),
        Err(e) => error!("trying to write message to log: {:?}", e),
    }
}

fn process_message(event: &Event) {
    match event {
        Event::Message(m) => {
            println!("#{}::{}> {}", m.provider_name, m.username, m.message)
        }
    }
}
