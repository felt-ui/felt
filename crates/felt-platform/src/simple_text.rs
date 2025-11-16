use std::sync::{Arc, OnceLock};
use skrifa::{MetadataProvider, raw::{FileRef, FontRef}};
use vello::kurbo::Affine;
use vello::peniko::{Blob, Brush, Fill, FontData, color::palette};
use vello::{Glyph, Scene};

const ROBOTO_FONT: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

static ROBOTO_FONT_DATA: OnceLock<FontData> = OnceLock::new();

fn get_roboto_font() -> &'static FontData {
    ROBOTO_FONT_DATA.get_or_init(|| {
        FontData::new(Blob::new(Arc::new(ROBOTO_FONT)), 0)
    })
}

pub struct SimpleText;

impl SimpleText {
    pub fn new() -> Self {
        Self
    }

    pub fn add(
        &mut self,
        scene: &mut Scene,
        _font: Option<&FontData>,
        size: f32,
        brush: Option<&Brush>,
        transform: Affine,
        text: &str,
    ) {
        let font = get_roboto_font();
        let brush = brush.unwrap_or(&Brush::Solid(palette::css::WHITE));

        let font_ref = to_font_ref(font).unwrap();
        let font_size = skrifa::instance::Size::new(size);
        let axes = font_ref.axes();
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations.iter().copied());
        let charmap = font_ref.charmap();
        let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);

        let mut pen_x = 0_f32;

        scene
            .draw_glyphs(font)
            .font_size(size)
            .transform(transform)
            .normalized_coords(bytemuck::cast_slice(var_loc.coords()))
            .brush(brush)
            .draw(
                Fill::NonZero,
                text.chars().map(|ch| {
                    let gid = charmap.map(ch).unwrap_or_default();
                    let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                    let x = pen_x;
                    pen_x += advance;
                    Glyph {
                        id: gid.to_u32(),
                        x,
                        y: 0.0,
                    }
                }),
            );
    }
}

impl Default for SimpleText {
    fn default() -> Self {
        Self::new()
    }
}

fn to_font_ref(font: &FontData) -> Option<FontRef<'_>> {
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}
