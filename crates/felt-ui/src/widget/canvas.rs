use crate::draw::{Affine, BlendMode, Brush, FillRule, Point, Rect, Size, StrokeStyle};
use crate::{PaintCtx, Widget};
use smallvec::SmallVec;
use vello::Scene;
use vello::kurbo::Shape;
use vello::peniko::Mix;

pub struct DrawContext<'a> {
    ctx: &'a mut PaintCtx,
    scene: &'a mut Scene,
    pub size: Size,
}

impl<'a> DrawContext<'a> {
    pub fn new(ctx: &'a mut PaintCtx, scene: &'a mut Scene, size: Size) -> Self {
        Self { ctx, scene, size }
    }

    pub fn fill(&mut self, fill_rule: FillRule, brush: &Brush, shape: &impl Shape) {
        let vello_brush = brush.to_vello();
        self.scene.fill(
            fill_rule.to_vello(),
            self.ctx.transform,
            &vello_brush,
            None,
            shape,
        );
    }

    pub fn stroke(&mut self, style: &StrokeStyle, brush: &Brush, shape: &impl Shape) {
        let vello_brush = brush.to_vello();
        let vello_stroke = style.to_vello();
        self.scene
            .stroke(&vello_stroke, self.ctx.transform, &vello_brush, None, shape);
    }

    pub fn push_layer(
        &mut self,
        blend: BlendMode,
        alpha: f32,
        transform: Affine,
        shape: &impl Shape,
    ) {
        // Combine the context transform with the pushed transform
        let combined_transform = self.ctx.transform * transform;
        self.scene
            .push_layer(blend.to_vello(), alpha, combined_transform, shape);
    }

    pub fn pop_layer(&mut self) {
        self.scene.pop_layer();
    }

    pub fn draw_image(&mut self, image: &crate::draw::Image, transform: Affine) {
        let combined_transform = self.ctx.transform * transform;
        self.scene.draw_image(image.to_vello(), combined_transform);
    }

    pub fn push_clip_layer(&mut self, shape: &impl Shape) {
        // For clip layers, we need to transform the shape to global coordinates
        let global_clip = self.ctx.transform.transform_rect_bbox(shape.bounding_box());
        self.scene.push_layer(
            vello::peniko::Mix::Clip,
            1.0,
            vello::kurbo::Affine::IDENTITY,
            &global_clip,
        );
    }

    pub fn append(&mut self, canvas: &Canvas, transform: Affine) {
        let combined_transform = self.ctx.transform * transform;
        self.scene.append(&canvas.scene, Some(combined_transform));
    }
}

pub struct Canvas {
    pub size: Size,
    pub painter: Box<dyn FnMut(&mut DrawContext)>,
    scene: Scene, // Internal scene for recording
}

impl Canvas {
    pub fn new(size: Size, painter: impl FnMut(&mut DrawContext) + 'static) -> Self {
        Self {
            size,
            painter: Box::new(painter),
            scene: Scene::new(),
        }
    }

    pub fn get_scene(&self) -> &Scene {
        &self.scene
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
