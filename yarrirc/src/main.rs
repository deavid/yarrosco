use anyhow::{Ok, Result};
use futures::prelude::*;
use irc::client::prelude::*;
use yarrcfg::parse_config;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    parse_config()?;
    return Ok(());
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("the-irc-crate".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        channels: vec!["#test-deavid".to_owned()],
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        println!("{:?}", message);
    }

    Ok(())
}
