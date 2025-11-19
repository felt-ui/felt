use crate::{PaintCtx, Widget};
use smallvec::SmallVec;
use vello::Scene;
use vello::kurbo::{Affine, Point, Rect, Shape, Size, Stroke};
use vello::peniko::{Brush, Fill, Mix};

pub struct DrawContext<'a> {
    ctx: &'a mut PaintCtx,
    scene: &'a mut Scene,
    pub size: Size,
}

impl<'a> DrawContext<'a> {
    pub fn new(ctx: &'a mut PaintCtx, scene: &'a mut Scene, size: Size) -> Self {
        Self { ctx, scene, size }
    }

    pub fn fill(&mut self, style: Fill, brush: &Brush, shape: &impl Shape) {
        self.scene
            .fill(style, self.ctx.transform, brush, None, shape);
    }

    pub fn stroke(&mut self, style: &Stroke, brush: &Brush, shape: &impl Shape) {
        self.scene
            .stroke(style, self.ctx.transform, brush, None, shape);
    }

    pub fn push_layer(&mut self, blend: Mix, alpha: f32, transform: Affine, shape: &impl Shape) {
        // Combine the context transform with the pushed transform
        let combined_transform = self.ctx.transform * transform;
        self.scene
            .push_layer(blend, alpha, combined_transform, shape);
    }

    pub fn pop_layer(&mut self) {
        self.scene.pop_layer();
    }
}

pub struct Canvas {
    pub size: Size,
    pub painter: Box<dyn FnMut(&mut DrawContext)>,
}

impl Canvas {
    pub fn new(size: Size, painter: impl FnMut(&mut DrawContext) + 'static) -> Self {
        Self {
            size,
            painter: Box::new(painter),
        }
    }
}

impl Widget for Canvas {
    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let rect = Rect::from_origin_size(Point::ORIGIN, self.size);

        // Clip to the canvas size
        // We must transform the local rect to global coordinates for the clip to work correctly
        let global_clip = ctx.transform.transform_rect_bbox(rect);
        scene.push_layer(Mix::Normal, 1.0, Affine::IDENTITY, &global_clip);

        let mut draw_ctx = DrawContext::new(ctx, scene, self.size);
        (self.painter)(&mut draw_ctx);

        scene.pop_layer();
    }

    fn children(&self) -> SmallVec<[crate::EntityId; 4]> {
        SmallVec::new()
    }
}

// Declarative API
use crate::element::Element;

pub struct CanvasElement {
    size: Size,
    painter: Option<Box<dyn FnMut(&mut DrawContext)>>,
}

impl CanvasElement {
    pub fn new(painter: impl FnMut(&mut DrawContext) + 'static) -> Self {
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

pub fn canvas(painter: impl FnMut(&mut DrawContext) + 'static) -> CanvasElement {
    CanvasElement::new(painter)
}
