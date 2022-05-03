use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use log::debug;
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

    let now = Instant::now();
    // Render an SVG
    let bottom = 700;
    let msgsz = data.len();
    let mut document = Document::new().set("viewBox", (0, 0, 250, bottom));
    // document = document.add(path)
    // document = document.add(text);
    let mut count = 0;
    for (n, e) in data.iter().enumerate().skip(10) {
        match e {
            yarrdata::Event::Message(m) => {
                // TODO: This doesn't wrap lines - we need to compute the widrth
                let text = format!("{}: {}", m.username, m.message);
                let mp = msgsz - n;
                let y = bottom as f64 - (mp as f64 * 10.0);
                let text = create_text(5.0, y, &text);
                document = document.add(text);
                count += 1;
            }
        }
    }
    println!("Craft: {:?} Count: {}", now.elapsed(), count);
    let now = Instant::now();

    svg::save(OUT_FILENAME, &document).unwrap();
    println!("Save: {:?}", now.elapsed());

    // let svgstr = document.to_string();
    // test_usvg_bboxes(&svgstr);
    test_fontdb();
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
    .set("style", "font-size:6px;font-weight:800; font-family:Sans;fill:#ffffff;stroke:#001338;stroke-width:1;paint-order:markers stroke fill")
    .set("x", x)
    .set("y", y)
    .set("id", id)
    .add(node::Text::new(text))
}

#[allow(dead_code)]
fn test_fontdb() {
    let mut db = fontdb::Database::new();
    let now = std::time::Instant::now();
    db.load_system_fonts();
    // db.set_serif_family("Times New Roman");
    // db.set_sans_serif_family("Arial");
    // db.set_cursive_family("Comic Sans MS");
    // db.set_fantasy_family("Impact");
    // db.set_monospace_family("Courier New");
    println!(
        "Loaded {} font faces in {}ms.",
        db.len(),
        now.elapsed().as_millis()
    );
    const FAMILY_NAME: &str = "DejaVu Sans";
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(FAMILY_NAME), fontdb::Family::SansSerif],
        weight: fontdb::Weight::BOLD,
        ..fontdb::Query::default()
    };

    let now = std::time::Instant::now();
    match db.query(&query) {
        Some(id) => {
            let (src, index) = db.face_source(id).unwrap();
            if let fontdb::Source::File(ref path) = &src {
                println!(
                    "Font '{}':{} found in {}ms.",
                    path.display(),
                    index,
                    now.elapsed().as_micros() as f64 / 1000.0
                );
                test_ttfparser(path);
            }
        }
        None => {
            println!("Error: '{}' not found.", FAMILY_NAME);
        }
    }
}
#[allow(dead_code)]
fn test_ttfparser(path: &PathBuf) {
    let font_data = std::fs::read(path).unwrap();

    let now = std::time::Instant::now();
    // TODO: ttf_parser::Face is referencing &font_data, so both must have compatible lifetimes.
    // ... this will impose a problem later as we will not be able to have the parser loaded permanently.
    // An option is looping from 0..face.number_of_glyphs(), craft GlyphId.0=i, and retrieve all glyphs.
    // By doing this we could cache the widhs into a BHashMap
    // Still there's no API to retrieve all mappings from char -> glyphId, so we will need to
    // use the source code of glyph_index and reimplement it ourselves.
    let face = match ttf_parser::Face::from_slice(&font_data, 0) {
        Ok(f) => f,
        Err(e) => {
            eprint!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let family_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    let post_script_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    println!("Family name: {:?}", family_name);
    println!("PostScript name: {:?}", post_script_name);
    println!("Units per EM: {:?}", face.units_per_em());
    println!("Ascender: {}", face.ascender());
    println!("Descender: {}", face.descender());
    println!("Line gap: {}", face.line_gap());
    println!("Global bbox: {:?}", face.global_bounding_box());
    println!("Number of glyphs: {}", face.number_of_glyphs());
    println!("Underline: {:?}", face.underline_metrics());
    println!("X height: {:?}", face.x_height());
    println!("Weight: {:?}", face.weight());
    println!("Width: {:?}", face.width());
    println!("Regular: {}", face.is_regular());
    println!("Italic: {}", face.is_italic());
    println!("Bold: {}", face.is_bold());
    println!("Oblique: {}", face.is_oblique());
    println!("Strikeout: {:?}", face.strikeout_metrics());
    println!("Subscript: {:?}", face.subscript_metrics());
    println!("Superscript: {:?}", face.superscript_metrics());
    println!("Variable: {:?}", face.is_variable());
    // TODO: Use this to calculate what fits in a line.
    // There is still the problem on how to translate these measures into a SVG
    // with a font-face of 6px.
    // Most probably: width / face.units_per_em() * 6(px)
    let test_string = "We are here!";
    for ch in test_string.chars() {
        let glyph = face.glyph_index(ch).unwrap();
        let width = face.glyph_hor_advance(glyph);
        println!("Char {:?} width: {:?}", ch, width);
    }

    println!("Elapsed: {}us", now.elapsed().as_micros());
}
#[allow(dead_code)]
fn test_usvg_bboxes(svgstr: &str) {
    use usvg::NodeExt;
    let now = Instant::now();
    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();

    println!("LoadFonts: {:?}", now.elapsed());
    let now = Instant::now();
    // FIXME: This is not a good idea; this takes a long time to convert the SVG.
    // usvg/src/text/mod.rs:: text_to_paths seems to contain the code to draw them, but can't understand it yet.
    let rtree = usvg::Tree::from_str(svgstr, &opt.to_ref()).unwrap();
    println!("Parsing: {:?}", now.elapsed());
    let now = Instant::now();
    for node in rtree.root().descendants() {
        if !rtree.is_in_defs(&node) {
            if let Some(bbox) = node.calculate_bbox().and_then(|r| r.to_rect()) {
                let b = format!(
                    "{:>5.1}, {:>5.1}, {:>5.1}, {:>5.1}",
                    bbox.left(),
                    bbox.right(),
                    bbox.top(),
                    bbox.bottom()
                );
                debug!("{:?} ?> {}", node.id(), b);
            }

            // Text bboxes are different from path bboxes.
            if let usvg::NodeKind::Path(ref path) = *node.borrow() {
                if let Some(ref bbox) = path.text_bbox {
                    let b = format!(
                        "{:>5.1}, {:>5.1}, {:>5.1}, {:>5.1}",
                        bbox.left(),
                        bbox.right(),
                        bbox.top(),
                        bbox.bottom()
                    );
                    debug!("{:?} #> {}", node.id(), b);
                }
            }
        }
    }
    println!("Bbox: {:?}", now.elapsed());
}
