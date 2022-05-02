use std::time::SystemTime;

use anyhow::Result;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::StreamExt;
use yarrdata::{Event, Message, ProviderQueue};
fn main() -> Result<()> {
    // TODO: Move initizalization to yarrcommon
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();
    // --
    let mut p = ProviderQueue::new("test-provider".to_owned());
    let e = Event::Message(Message {
        provider_name: "test-irc".to_owned(),
        room: "#test".to_owned(),
        message: "todo!()".to_owned(),
        username: "myself".to_owned(),
        msgid: "1234".to_owned(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        ..Default::default()
    });
    p.publish_sync(e.clone())?;
    p.publish_sync(e.clone())?;
    let mut s = p.subscribe();
    let ms = &mut s;
    block_on(async move {
        dbg!(ms.next().await);
        // Blocks until the next message is ready.
        dbg!(ms.next().await);
    });
    let ms = &mut s;
    // This one doesn't block, but consumes the item. .peekable() can be used
    // if we don't want to consume.
    dbg!(ms.next().now_or_never());
    p.publish_sync(e)?;
    dbg!(ms.next().now_or_never());
    dbg!(ms.next().now_or_never());
    p.close_sync()?;
    // Now it returns Some(None) from here on to signal the publisher has closed the stream.
    dbg!(ms.next().now_or_never());
    dbg!(ms.next().now_or_never());
    Ok(())
}
