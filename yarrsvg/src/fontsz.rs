use std::path::PathBuf;

use log::{debug, warn};

pub struct Font {
    face: ttf_parser::Face<'static>,
    pub size: f64,
}

impl Font {
    pub fn new(path: PathBuf) -> Self {
        let font_data = std::fs::read(path).unwrap();
        let font_lk: &'static [u8] = Box::leak(font_data.into_boxed_slice());
        let face = match ttf_parser::Face::from_slice(font_lk, 0) {
            Ok(f) => f,
            Err(e) => {
                eprint!("Error: {}.", e);
                std::process::exit(1);
            }
        };
        Self { face, size: 1.0 }
    }
    #[allow(dead_code)]
    pub fn print_info(&self) {
        let family_name = self
            .face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode())
            .and_then(|name| name.to_string());

        let post_script_name = self
            .face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
            .and_then(|name| name.to_string());

        println!("Family name: {:?}", family_name);
        println!("PostScript name: {:?}", post_script_name);
        println!("Units per EM: {:?}", self.face.units_per_em());
        println!("Ascender: {}", self.face.ascender());
        println!("Descender: {}", self.face.descender());
        println!("Line gap: {}", self.face.line_gap());
        println!("Global bbox: {:?}", self.face.global_bounding_box());
        println!("Number of glyphs: {}", self.face.number_of_glyphs());
        println!("Underline: {:?}", self.face.underline_metrics());
        println!("X height: {:?}", self.face.x_height());
        println!("Weight: {:?}", self.face.weight());
        println!("Width: {:?}", self.face.width());
        println!("Regular: {}", self.face.is_regular());
        println!("Italic: {}", self.face.is_italic());
        println!("Bold: {}", self.face.is_bold());
        println!("Oblique: {}", self.face.is_oblique());
        println!("Strikeout: {:?}", self.face.strikeout_metrics());
        println!("Subscript: {:?}", self.face.subscript_metrics());
        println!("Superscript: {:?}", self.face.superscript_metrics());
        println!("Variable: {:?}", self.face.is_variable());
    }
    pub fn char_width(&self, text: &str) -> Vec<f64> {
        let mut total_width: Vec<f64> = vec![];
        let uem = self.face.units_per_em();
        for ch in text.chars() {
            if let Some(glyph) = self.face.glyph_index(ch) {
                if let Some(width) = self.face.glyph_hor_advance(glyph) {
                    debug!("Char {:?} width: {:?}", ch, width);
                    total_width.push(width as f64 * self.size / uem as f64);
                } else {
                    warn!("Char {:?} doesn't have width", ch);
                }
                // FIXME: There's some slight discrepancy between this and SVG rendered
                //   text. Might be a font mismatch or some setting, or the side bearings.
                //   All that we tested seems to be worse than nothing.
                // if let Some(bearing) = self.face.glyph_hor_side_bearing(glyph) {
                //     debug!("Char {:?} bearing: {:?}", ch, bearing);
                //     total_width.push(-bearing as f64 / uem as f64 * 0.0);
                // }
            } else {
                warn!("Char {:?} doesn't exist in font", ch);
            }
        }
        total_width
    }
    pub fn split_lines(&self, text: &str, maxwidth: f64) -> Vec<String> {
        let mut ret: Vec<String> = vec![];
        let uem = self.face.units_per_em();
        let mut xpos = 0.0;
        let mut line = String::new();

        for ch in text.chars() {
            if let Some(glyph) = self.face.glyph_index(ch) {
                if let Some(width) = self.face.glyph_hor_advance(glyph) {
                    let width: f64 = width as f64 / uem as f64 * self.size;
                    if xpos + width < maxwidth {
                        line.push(ch);
                        xpos += width;
                    } else {
                        ret.push(line.clone());
                        line.clear();
                        line.push(ch);
                        xpos = width;
                    }
                }
            }
        }
        ret.push(line);

        ret
    }
}

pub struct FontDB {
    pub db: fontdb::Database,
}

impl FontDB {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        // db.set_serif_family("Times New Roman");
        // db.set_sans_serif_family("Arial");
        // db.set_cursive_family("Comic Sans MS");
        // db.set_fantasy_family("Impact");
        // db.set_monospace_family("Courier New");
        debug!(
            "Loaded {} font faces in {}ms.",
            db.len(),
            now.elapsed().as_millis()
        );
        FontDB { db }
    }
    pub fn query(&self, family_name: &str) -> Option<PathBuf> {
        let now = std::time::Instant::now();
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family_name), fontdb::Family::SansSerif],
            weight: fontdb::Weight::BOLD,
            ..fontdb::Query::default()
        };
        match self.db.query(&query) {
            Some(id) => {
                let (src, index) = self.db.face_source(id).unwrap();
                if let fontdb::Source::File(ref path) = &src {
                    debug!(
                        "Font '{}':{} found in {}ms.",
                        path.display(),
                        index,
                        now.elapsed().as_micros() as f64 / 1000.0
                    );
                    return Some(path.clone());
                } else {
                    warn!("no font path found for {:?}", family_name);
                }
            }
            None => {
                warn!("Error: {:?} not found.", family_name);
            }
        }
        None
    }
}
