use crate::{PaintCtx, Widget};
use smallvec::SmallVec;
use vello::Scene;
use vello::kurbo::{Rect, Size};

pub struct Canvas {
    pub size: Size,
    pub painter: Box<dyn FnMut(&mut PaintCtx, &mut Scene)>,
}

impl Canvas {
    pub fn new(size: Size, painter: impl FnMut(&mut PaintCtx, &mut Scene) + 'static) -> Self {
        Self {
            size,
            painter: Box::new(painter),
        }
    }
}

impl Widget for Canvas {
    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        (self.painter)(ctx, scene);
    }

    fn children(&self) -> SmallVec<[crate::EntityId; 4]> {
        SmallVec::new()
    }
}

// Declarative API
use crate::element::Element;

pub struct CanvasElement {
    size: Size,
    painter: Option<Box<dyn FnMut(&mut PaintCtx, &mut Scene)>>,
}

impl CanvasElement {
    pub fn new(painter: impl FnMut(&mut PaintCtx, &mut Scene) + 'static) -> Self {
        Self {
            size: Size::ZERO,
            painter: Some(Box::new(painter)),
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl Element for CanvasElement {
    fn build(mut self: Box<Self>) -> Box<dyn Widget> {
        let painter = self.painter.take().unwrap();
        Box::new(Canvas::new(self.size, painter))
    }
}

pub fn canvas(painter: impl FnMut(&mut PaintCtx, &mut Scene) + 'static) -> CanvasElement {
    CanvasElement::new(painter)
}
