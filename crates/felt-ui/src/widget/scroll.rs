use crate::{EntityId, PaintCtx, Widget};
use smallvec::SmallVec;
use vello::Scene;
use vello::kurbo::{Affine, Point, Rect, Vec2};
use vello::peniko::Mix;

pub struct ScrollView {
    pub offset: Vec2,
    pub size: Vec2,
    pub child: Option<Box<dyn Widget>>,
}

impl ScrollView {
    pub fn new(size: Vec2) -> Self {
        Self {
            offset: Vec2::ZERO,
            size,
            child: None,
        }
    }

    pub fn with_child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Widget for ScrollView {
    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let viewport = Rect::from_origin_size(Point::ORIGIN, (self.size.x, self.size.y));

        // 1. Calculate Global Clip Rect
        // We transform the viewport rect by the current context transform to get the clip in scene coordinates.
        // This avoids relying on push_layer's transform behavior for clipping, which might be subtle.
        // Note: This assumes ctx.transform is only translation/scale, which it is.
        // For rotation, we'd need a Shape transform, but Rect transform is fine here.
        let global_clip = ctx.transform.transform_rect_bbox(viewport);

        // 2. Push Clip Layer
        // We use Identity transform for the layer, but provide the transformed clip rect.
        scene.push_layer(Mix::Normal, 1.0, Affine::IDENTITY, &global_clip);

        // 3. Paint Child with Manual Transform
        // We pass the full transform (Parent * ScrollOffset) to the child.
        if let Some(child) = &mut self.child {
            let mut child_ctx = PaintCtx {
                transform: ctx
                    .transform
                    .then_translate(Vec2::new(-self.offset.x, -self.offset.y)),
                clip: global_clip,
            };
            child.paint(&mut child_ctx, scene);
        }

        // 4. Pop Clip Layer
        scene.pop_layer();
    }

    fn children(&self) -> SmallVec<[EntityId; 4]> {
        SmallVec::new() // Stub for PoC
    }
}

// Declarative API
use crate::element::{Element, IntoElement};

pub struct ScrollViewElement {
    size: Vec2,
    offset: Vec2,
    child: Option<Box<dyn Widget>>,
}

impl ScrollViewElement {
    pub fn new() -> Self {
        Self {
            size: Vec2::ZERO,
            offset: Vec2::ZERO,
            child: None,
        }
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_element().build());
        self
    }
}

impl Element for ScrollViewElement {
    fn build(self: Box<Self>) -> Box<dyn Widget> {
        let mut sv = ScrollView::new(self.size);
        sv.offset = self.offset;
        if let Some(child) = self.child {
            sv.child = Some(child);
        }
        Box::new(sv)
    }
}

pub fn scroll_view() -> ScrollViewElement {
    ScrollViewElement::new()
}
