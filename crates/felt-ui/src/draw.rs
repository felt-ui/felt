// Drawing primitives module - abstracts Vello types
use vello::kurbo;
use vello::peniko;

// Re-export core geometric types
pub use kurbo::{
    Affine, Arc, BezPath, Circle, Ellipse, Line, Point, Rect, RoundedRect, Size, Vec2,
};

// Color abstraction
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub(crate) fn to_vello(&self) -> peniko::Color {
        peniko::Color::rgba8(self.r, self.g, self.b, self.a)
    }
}

// Brush abstraction
#[derive(Clone, Debug)]
pub enum Brush {
    Solid(Color),
    Gradient(Gradient),
}

impl Brush {
    pub(crate) fn to_vello(&self) -> peniko::Brush {
        match self {
            Brush::Solid(color) => peniko::Brush::Solid(color.to_vello()),
            Brush::Gradient(gradient) => gradient.to_vello(),
        }
    }
}

// Gradient abstraction
#[derive(Clone, Debug)]
pub struct Gradient {
    // Simplified for now - can be expanded
    pub(crate) inner: peniko::Gradient,
}

impl Gradient {
    pub(crate) fn to_vello(&self) -> peniko::Brush {
        peniko::Brush::Gradient(self.inner.clone())
    }
}

// Fill style
#[derive(Clone, Copy, Debug)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

impl FillRule {
    pub(crate) fn to_vello(&self) -> peniko::Fill {
        match self {
            FillRule::NonZero => peniko::Fill::NonZero,
            FillRule::EvenOdd => peniko::Fill::EvenOdd,
        }
    }
}

// Stroke style
#[derive(Clone, Debug)]
pub struct StrokeStyle {
    pub width: f64,
}

impl StrokeStyle {
    pub fn new(width: f64) -> Self {
        Self { width }
    }

    pub(crate) fn to_vello(&self) -> kurbo::Stroke {
        kurbo::Stroke::new(self.width)
    }
}

// Blend mode
#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Clip,
}

impl BlendMode {
    pub(crate) fn to_vello(&self) -> peniko::Mix {
        match self {
            BlendMode::Normal => peniko::Mix::Normal,
            BlendMode::Multiply => peniko::Mix::Multiply,
            BlendMode::Screen => peniko::Mix::Screen,
            BlendMode::Overlay => peniko::Mix::Overlay,
            BlendMode::Darken => peniko::Mix::Darken,
            BlendMode::Lighten => peniko::Mix::Lighten,
            BlendMode::ColorDodge => peniko::Mix::ColorDodge,
            BlendMode::ColorBurn => peniko::Mix::ColorBurn,
            BlendMode::HardLight => peniko::Mix::HardLight,
            BlendMode::SoftLight => peniko::Mix::SoftLight,
            BlendMode::Difference => peniko::Mix::Difference,
            BlendMode::Exclusion => peniko::Mix::Exclusion,
            BlendMode::Hue => peniko::Mix::Hue,
            BlendMode::Saturation => peniko::Mix::Saturation,
            BlendMode::Color => peniko::Mix::Color,
            BlendMode::Luminosity => peniko::Mix::Luminosity,
            BlendMode::Clip => peniko::Mix::Clip,
        }
    }
}

// Image abstraction
#[derive(Clone)]
pub struct Image {
    pub(crate) inner: peniko::Image,
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        let inner = peniko::Image::new(
            peniko::Blob::new(std::sync::Arc::new(data)),
            peniko::Format::Rgba8,
            width,
            height,
        );
        Self { inner }
    }

    pub fn from_vello(inner: peniko::Image) -> Self {
        Self { inner }
    }

    pub(crate) fn to_vello(&self) -> &peniko::Image {
        &self.inner
    }
}
