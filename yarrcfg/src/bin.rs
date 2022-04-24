use anyhow::Result;
use yarrcfg::parse_config;
fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    let cfg = parse_config()?;
    println!("{:#?}", cfg);
    Ok(())
}
