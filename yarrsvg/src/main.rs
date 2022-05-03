use anyhow::Result;
use log::error;
use log::LevelFilter;
use svg::node;
use svg::Document;
use yarrdata::db;

const OUT_FILENAME: &str = "./yarrosco_chat.svg";

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

    // Read from database
    let mut log = db::Log::new(100, cfg.logfile, cfg.checkpointfile);
    if let Err(e) = log.load().await {
        error!("couldn't load the database: {:?}", e);
    }
    let data: Vec<yarrdata::Event> = log.data.values().map(|ce| ce.event.clone()).collect();

    // let data = element::path::Data::new()
    //     .move_to((10, 10))
    //     .line_by((0, 40))
    //     .line_by((50, 0))
    //     .line_by((0, -50))
    //     .close();

    // let path = node::element::Path::new()
    //     .set("fill", "none")
    //     .set("stroke", "black")
    //     .set("stroke-width", 3)
    //     .set("d", data);

    // let text = node::element::Text::new()
    //     .set("xml_space", "preserve")
    //     .set("style", "font-size:6px;font-family:Sans;fill:#dfe6ff;stroke:#001338;stroke-width:1;paint-order:markers stroke fill")
    //     .set("id", "text_1234")
    //     .set("x", "15")
    //     .set("y", "30")
    //     .add(node::Text::new("content"));

    // Render an SVG
    let bottom = 700;
    let msgsz = data.len();
    let mut document = Document::new().set("viewBox", (0, 0, 250, bottom));
    // document = document.add(path)
    // document = document.add(text);

    for (n, e) in data.iter().enumerate() {
        match e {
            yarrdata::Event::Message(m) => {
                // TODO: This doesn't wrap lines - we need to compute the widrth
                let text = format!("{}: {}", m.username, m.message);
                let mp = msgsz - n;
                let y = bottom as f64 - (mp as f64 * 10.0);
                let text = create_text(5.0, y, &text);
                document = document.add(text);
            }
        }
    }

    svg::save(OUT_FILENAME, &document).unwrap();
    Ok(())
}

fn create_text(x: f64, y: f64, text: &str) -> node::element::Text {
    let text = text
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let x = format!("{:.2}", x);
    let y = format!("{:.2}", y);
    node::element::Text::new()
    .set("xml_space", "preserve")
    .set("style", "font-size:6px;font-weight:800; font-family:Sans;fill:#ffffff;stroke:#001338;stroke-width:1;paint-order:markers stroke fill")
    .set("x", x)
    .set("y", y)
    .add(node::Text::new(text))
}
