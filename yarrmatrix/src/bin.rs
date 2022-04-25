use anyhow::Result;
use log::{debug, LevelFilter};
use matrix_sdk::{
    room::Room,
    ruma::events::{room::message::MessageEventContent, SyncMessageEvent},
    ruma::UserId,
    Client, SyncSettings,
};
use ruma_identifiers::DeviceId;
use yarrcfg::parse_config;

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
    let matrix = cfg.matrix.expect("Matrix config is needed to run IRC");

    let user = UserId::try_from(matrix.user_id.clone())?;
    let client = Client::new_from_user_id(user.clone()).await?;
    let session = matrix_sdk::Session {
        access_token: matrix.access_token.0.clone(),
        user_id: user,
        device_id: DeviceId::new(),
    };

    let target_room = matrix.room_id.clone();
    debug!("authenticating as {:?}", &matrix.user_id);
    client.restore_login(session).await?;
    debug!("waiting for messages");
    client
        .register_event_handler(
            move |ev: SyncMessageEvent<MessageEventContent>, room: Room, _client: Client| {
                let target_room = target_room.clone();
                async move {
                    let room_id = room.room_id().as_str();
                    if target_room != room_id {
                        debug!("Ignored message from room ID {:?}", room_id)
                    } else {
                        println!("Room: n:{:?} id:{:?}", room.name(), room.room_id());
                        println!("Received a message {:?}", ev);
                    }
                }
            },
        )
        .await;

    // Syncing is important to synchronize the client state with the server.
    // This method will never return.
    client.sync(SyncSettings::default()).await;

    Ok(())
}
