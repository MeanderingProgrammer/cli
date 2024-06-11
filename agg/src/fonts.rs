use resvg::usvg::fontdb::{Database, Family, Query, Stretch, Style, Weight};

pub fn init(font_dirs: &[String], fonts: &[String]) -> Option<(Database, Vec<String>)> {
    let mut font_db = Database::new();
    font_db.load_system_fonts();
    for dir in font_dirs {
        font_db.load_fonts_dir(shellexpand::tilde(dir).to_string());
    }

    let mut families: Vec<String> = fonts
        .iter()
        .map(|name| name.trim())
        .filter_map(|name| find_font_family(&font_db, name))
        .collect();

    if families.is_empty() {
        None
    } else {
        ["DejaVu Sans", "Noto Emoji"]
            .into_iter()
            .filter_map(|name| find_font_family(&font_db, name))
            .for_each(|family| families.push(family));
        Some((font_db, families))
    }
}

fn find_font_family(font_db: &Database, name: &str) -> Option<String> {
    let family = Family::Name(name);

    let query = Query {
        families: &[family],
        weight: Weight::NORMAL,
        stretch: Stretch::Normal,
        style: Style::Normal,
    };

    font_db.query(&query).and_then(|face_id| {
        let face_info = font_db.face(face_id).unwrap();
        face_info.families.first().map(|(family, _)| family.clone())
    })
}
