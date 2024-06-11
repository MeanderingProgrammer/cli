use fontdue::{Font, FontSettings, Metrics};
use resvg::usvg::fontdb::{Database, Family, Query, Stretch, Style, Weight, ID};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct Variant {
    pub bold: bool,
    pub italic: bool,
}

impl Variant {
    fn weight(&self) -> Weight {
        if self.bold {
            Weight::BOLD
        } else {
            Weight::NORMAL
        }
    }

    fn style(&self) -> Style {
        if self.italic {
            Style::Italic
        } else {
            Style::Normal
        }
    }
}

type FontVariant = (String, Variant);
type CharVariant = (char, Variant);
type Glyph = (Metrics, Vec<u8>);

#[derive(Debug)]
pub struct CachingFontDb {
    pub db: Database,
    font_cache: HashMap<FontVariant, Option<Font>>,
    glyph_cache: HashMap<CharVariant, Option<Glyph>>,
}

impl CachingFontDb {
    pub fn new(font_dirs: &[String]) -> Self {
        let mut db = Database::new();
        db.load_system_fonts();
        for dir in font_dirs {
            db.load_fonts_dir(shellexpand::tilde(dir).to_string());
        }
        Self {
            db,
            font_cache: HashMap::new(),
            glyph_cache: HashMap::new(),
        }
    }

    pub fn available_fonts(&self, fonts: &[String]) -> Vec<String> {
        let mut families: Vec<String> = fonts
            .iter()
            .map(|name| name.trim())
            .filter_map(|name| self.get_font_family(name))
            .collect();
        if !families.is_empty() {
            ["DejaVu Sans", "Noto Emoji"]
                .into_iter()
                .filter_map(|name| self.get_font_family(name))
                .for_each(|family| families.push(family));
        }
        families
    }

    fn get_font_family(&self, name: &str) -> Option<String> {
        let font_id = self.get_id(&[name], &Variant::default())?;
        log::debug!("found font with id={:?}", font_id);
        let font_info = self.db.face(font_id)?;
        font_info.families.first().map(|(family, _)| family.clone())
    }

    pub fn get_font(&self, families: &[&str], variant: &Variant) -> Option<Font> {
        let font_id = self.get_id(families, variant)?;
        log::debug!("found font with id={:?}", font_id);
        self.db.with_face_data(font_id, |font_data, face_index| {
            let settings = FontSettings {
                collection_index: face_index,
                ..Default::default()
            };
            Font::from_bytes(font_data, settings).unwrap()
        })
    }

    pub fn get_glyph_cache(
        &mut self,
        key: CharVariant,
        font_size: f32,
        font_families: &[String],
    ) -> &Option<Glyph> {
        if !self.glyph_cache.contains_key(&key) {
            let rasterized = match self.rasterize_glyph(key.clone(), font_size, font_families) {
                Some(glyph) => Some(glyph),
                None => {
                    if key.1.bold || key.1.italic {
                        self.rasterize_glyph((key.0, Variant::default()), font_size, font_families)
                    } else {
                        None
                    }
                }
            };
            self.glyph_cache.insert(key.clone(), rasterized);
        }
        &self.glyph_cache[&key]
    }

    fn rasterize_glyph(
        &mut self,
        key: CharVariant,
        font_size: f32,
        font_families: &[String],
    ) -> Option<Glyph> {
        font_families.iter().cloned().find_map(|name| {
            match self.get_font_cache((name, key.1.clone())) {
                Some(font) => {
                    let idx = font.lookup_glyph_index(key.0);
                    if idx > 0 {
                        Some(font.rasterize_indexed(idx, font_size))
                    } else {
                        None
                    }
                }
                None => None,
            }
        })
    }

    fn get_font_cache(&mut self, key: FontVariant) -> &Option<Font> {
        if !self.font_cache.contains_key(&key) {
            let font = self.get_font(&[&key.0], &key.1);
            self.font_cache.insert(key.clone(), font);
        }
        &self.font_cache[&key]
    }

    fn get_id(&self, families: &[&str], variant: &Variant) -> Option<ID> {
        let weight = variant.weight();
        let style = variant.style();
        log::debug!(
            "looking up font for families={:?}, weight={}, style={:?}",
            families,
            weight.0,
            style
        );
        let families: Vec<Family> = families.iter().map(|name| Family::Name(name)).collect();
        let query = Query {
            families: &families,
            stretch: Stretch::Normal,
            weight,
            style,
        };
        self.db.query(&query)
    }
}
