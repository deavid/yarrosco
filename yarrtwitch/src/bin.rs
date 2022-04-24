use anyhow::{Ok, Result};
use futures::prelude::*;
use irc::client::prelude::*;
use log::{debug, error, info, warn};
use yarrcfg::parse_config;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    let cfg = parse_config()?;
    let twitch = cfg.twitch.expect("Twitch config is needed to run IRC");

    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some(twitch.username.clone()),
        server: Some(twitch.server()),
        port: twitch.port()?,
        channels: twitch.channels,
        password: Some(format!("oauth:{}", &twitch.oauth_token.0)),
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;
    let extensions: Vec<Capability> = vec![Capability::Custom(":twitch.tv/tags")];
    client.send_cap_req(&extensions)?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        // match message.prefix.unwrap() {
        //     Prefix::ServerName(_) => todo!(),
        //     Prefix::Nickname(_nick, _user, _host) => todo!(),
        // }
        match &message.command {
            Command::PING(_, _) => {}
            Command::PONG(_, _) => {}

            Command::NOTICE(tgt, msg) => {
                info!("NOTICE[{}]: {}", tgt, msg);
            }
            Command::Response(r, data) => match r.is_error() {
                true => error!("E[{:?}] < {:?}", r, data),
                false => {
                    match r {
                        Response::RPL_MOTD | Response::RPL_MOTDSTART => {}
                        Response::RPL_ENDOFMOTD => info!("IRC Connection successful."),
                        _ => debug!("< [{:?}] {:?}", r, data)
                    };
                }
            },
            Command::PRIVMSG(tgt, msg) => info!(
                "P.Msg[{}]: {} (pfix: {:?} tags: {:?})",
                tgt, msg, &message.prefix, &message.tags
            ),
            c => debug!(": {:?}", c),
        }
    }
    // TODO: Sometimes, specially when connecting, it kills the connection before auth.
    warn!("IRC Connection ended");
    Ok(())
}
