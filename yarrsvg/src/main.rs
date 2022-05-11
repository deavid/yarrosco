mod fontsz;
use std::time::Instant;

use anyhow::Result;
use log::error;
use log::info;
use log::LevelFilter;
use svg::node;
use svg::node::element;
use svg::node::element::Definitions;
use svg::node::element::Element;
use svg::node::element::Filter;
use svg::node::element::Group;
use svg::Document;
use svg::Node;
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

    let now = Instant::now();

    let fdb = fontsz::FontDB::new();
    let dejavu_path = fdb.query("Noto Sans").expect("error retrieving font");
    let mut dejavu_face = fontsz::Font::new(dejavu_path);
    // dejavu_face.print_info();
    dejavu_face.size = 18.0;
    // Render an SVG
    let doc_height: f64 = 700.0;
    let doc_width: f64 = 700.0;
    let margin: f64 = 20.0;
    let mut document = Document::new().set(
        "viewBox",
        (
            0.0_f64,
            0.0_f64,
            doc_width + margin * 2.0,
            doc_height + margin * 2.0,
        ),
    );
    let mut myfilter = Filter::new()
        .set("id", "MyFilter")
        .set("filterUnits", "userSpaceOnUse")
        .set("x", 0)
        .set("y", 0)
        .set("width", doc_width + margin * 2.0)
        .set("height", doc_height + margin * 2.0);
    let mut blur = Element::new("feGaussianBlur");
    blur.assign("in", "SourceAlpha");
    blur.assign("stdDeviation", "8");
    blur.assign("result", "blur");

    let mut merge = Element::new("feMerge");
    let mut mergenode = Element::new("feMergeNode");
    mergenode.assign("in", "blur");
    merge.append(mergenode);
    let mut mergenode = Element::new("feMergeNode");
    mergenode.assign("in", "blur");
    merge.append(mergenode);
    let mut mergenode = Element::new("feMergeNode");
    mergenode.assign("in", "blur");
    merge.append(mergenode);
    let mut mergenode = Element::new("feMergeNode");
    mergenode.assign("in", "SourceGraphic");
    merge.append(mergenode);

    myfilter = myfilter.add(blur).add(merge);
    let defs = Definitions::new().add(myfilter);
    document = document.add(defs);

    //     <defs>
    //     <filter id="MyFilter" filterUnits="userSpaceOnUse" x="0" y="0" width="710" height="700">
    //       <feGaussianBlur in="SourceAlpha" stdDeviation="8" result="blur"/>
    //       <feMerge>
    //         <feMergeNode in="blur"/>
    //         <feMergeNode in="blur"/>
    //         <feMergeNode in="blur"/>
    //         <feMergeNode in="SourceGraphic"/>
    //       </feMerge>
    //     </filter>
    //   </defs>
    //  <g filter="url(#MyFilter)" >

    let mut main_group = Group::new().set("filter", "url(#MyFilter)");

    // document = document.add(path)
    // document = document.add(text);
    let mut count = 0;
    let mut n = 0.0;
    for e in data.iter().rev() {
        match e {
            yarrdata::Event::Message(m) => {
                // TODO: This doesn't wrap lines - we need to compute the width
                let text = format!("{}: {}", m.username, m.message);
                let y = doc_height as f64 - (n * dejavu_face.size * 1.2) - margin;
                if y < 0.0 {
                    break;
                }

                let lines = dejavu_face.split_lines(&text, doc_width as f64);
                for text in lines.iter().rev() {
                    let y = doc_height as f64 - (n * dejavu_face.size * 1.2) - margin;
                    n += 1.0;
                    let width: Vec<f64> = dejavu_face.char_width(text);
                    let sumwidth: f64 = width.iter().sum();
                    info!("w: {:.2}px, t: {}", sumwidth, &text);
                    let text = create_text(margin, y, text);
                    // let mut xpos: f64 = 5.0;
                    // for w in width.iter().copied() {
                    //     let data = element::path::Data::new()
                    //         .move_to((xpos, y + 1.0))
                    //         .line_by((w, 1.0))
                    //         .close();
                    //     let path = node::element::Path::new()
                    //         .set("fill", "none")
                    //         .set("stroke", "black")
                    //         .set("stroke-width", 0.5)
                    //         .set("d", data);
                    //     document = document.add(path);
                    //     xpos += w;
                    // }

                    // let data = element::path::Data::new()
                    //     .move_to((5.0 + sumwidth, y))
                    //     .line_by((0.0, -szpx))
                    //     .close();

                    // let path = node::element::Path::new()
                    //     .set("fill", "none")
                    //     .set("stroke", "black")
                    //     .set("stroke-width", 1)
                    //     .set("d", data);
                    // document = document.add(path);
                    main_group = main_group.add(text);
                }
                n += 0.5;

                count += 1;
            }
        }
    }
    document = document.add(main_group);

    info!("Craft: {:?} Count: {}", now.elapsed(), count);
    let now = Instant::now();

    svg::save(OUT_FILENAME, &document).unwrap();
    info!("Save: {:?}", now.elapsed());

    // let svgstr = document.to_string();
    // test_usvg_bboxes(&svgstr);
    Ok(())
}

fn create_text(x: f64, y: f64, text: &str) -> node::element::Text {
    let mut id = text
        .replace('"', "_")
        .replace(' ', "")
        .replace('&', "_")
        .replace('\'', "_")
        .replace('<', "_")
        .replace('>', "_")
        + "______________";
    let text = text
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    if id.len() > 30 {
        id = id[..30].to_owned();
    }
    let x = format!("{:.2}", x);
    let y = format!("{:.2}", y);
    node::element::Text::new()
    .set("xml_space", "preserve")
    .set("style", "font-size:18px;font-weight:700; font-family:'Noto Sans';fill:#ffffff;stroke:#001338;stroke-width:6;paint-order:markers stroke fill")
    .set("x", x)
    .set("y", y)
    .set("id", id)
    .add(node::Text::new(text))
}
