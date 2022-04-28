use anyhow::Result;
use futures::StreamExt;
use log::LevelFilter;
use std::borrow::Borrow;
use yarrcfg::parse_config;
use yarrdata::Event;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::builder()
        .filter(Some("sled"), LevelFilter::Info)
        .filter(Some("reqwest"), LevelFilter::Info)
        .init();

    let cfg = parse_config()?;
    let matrix_cfg = cfg.matrix.expect("Matrix config is needed to run IRC");
    let mut mx = yarrmatrix::MatrixClient::new(&matrix_cfg)?;

    let mut matrix_sub = mx.subscribe();
    let mx_future = tokio::task::spawn(async move { mx.run().await });

    while let Some(ev) = matrix_sub.next().await {
        // Upon receiving a new message...
        match ev.borrow() {
            Event::Message(m) => {
                println!(">> {:?}>> {}", std::thread::current().id(), m.message)
            }
        }
    }
    mx_future.await??;
    Ok(())
}
