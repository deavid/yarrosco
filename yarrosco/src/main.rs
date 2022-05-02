use anyhow::Result;
use futures::StreamExt;
use log::LevelFilter;
use log::{error, info};
use std::{borrow::Borrow, sync::Arc};
use tokio::sync::Mutex;
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
    let mut subs = vec![];
    let mut service_fut = vec![];
    // Create yarrtwitch
    for (_name, twitch_cfg) in cfg.twitch.iter() {
        let mut tw = TwitchClient::new(twitch_cfg).await?;
        // Subscribe to twitch
        subs.push(tw.subscribe());
        service_fut.push(task::spawn(async move { tw.run().await }));
    }

    // Create and connect to matrix
    for (_name, matrix_cfg) in cfg.matrix.iter() {
        let mut mx = MatrixClient::new(matrix_cfg)?;
        // Subscribe to matrix
        subs.push(mx.subscribe());
        service_fut.push(task::spawn(async move { mx.run().await }));
    }

    // Read from database
    let mut log = db::Log::new(100, cfg.logfile, cfg.checkpointfile);
    if let Err(e) = log.load().await {
        error!("couldn't load the database: {:?}", e);
    }
    for (_, ce) in log.data.iter() {
        process_message(&ce.event);
    }
    let log: Arc<Mutex<db::Log>> = Arc::new(Mutex::new(log));
    // TODO: Implement a yarrosco-secondary to have as a background + backup (name: yarrly? yarrdy? female-parrot)
    // Upon receiving a new matrix message...
    let futures_sub = subs.into_iter().map(|sub| {
        sub.for_each_concurrent(2, |event| async {
            process_message_log(Arc::clone(&log), event).await;
        })
    });
    use futures::stream::FuturesUnordered;
    let mut futures = FuturesUnordered::new();
    futures.extend(futures_sub);
    while futures.next().await.is_some() {}

    for fut in service_fut {
        fut.await??;
    }
    Ok(())
}

async fn process_message_log(logger: Arc<Mutex<db::Log>>, ev: Arc<Event>) {
    let event: &Event = ev.borrow();
    let mut logger_lck = logger.lock().await;
    let result = logger_lck.push(event.clone()).await;
    drop(logger_lck);
    match result {
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
