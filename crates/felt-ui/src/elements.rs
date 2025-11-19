use crate::Widget;
use crate::draw::Color;
use crate::element::{Element, IntoElement};
use crate::widget::container::Container;
use vello::kurbo::{Size, Vec2};

pub struct Div {
    child: Option<Box<dyn crate::Widget>>,
    size: Option<Size>,
    bg: Option<Color>,
    border: Option<(Color, f64)>,
    offset: Vec2,
}

impl Div {
    pub fn new() -> Self {
        Self {
            child: None,
            size: None,
            bg: None,
            border: None,
            offset: Vec2::ZERO,
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_element().build());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn border(mut self, color: Color, width: f64) -> Self {
        self.border = Some((color, width));
        self
    }

    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
}

impl Element for Div {
    fn build(self: Box<Self>) -> Box<dyn Widget> {
        Box::new(Container {
            child: self.child,
            background: self.bg,
            border: self.border,
            offset: self.offset,
            size: self.size,
        })
    }
}

pub fn div() -> Div {
    Div::new()
}
