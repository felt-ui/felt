use smallvec::SmallVec;
use vello::Scene;

pub mod draw;
pub mod element;
pub mod elements;
pub mod widget;

pub use draw::{
    Affine, BlendMode, Brush, Circle, Color, FillRule, Image, Line, Point, Rect, RoundedRect, Size,
    StrokeStyle, Vec2,
};
pub use element::{Element, IntoElement};
pub use elements::div;
pub use widget::canvas::{DrawContext, canvas};
pub use widget::scroll::scroll_view;

pub type EntityId = u64;

pub struct EventCtx;
pub struct LayoutCtx;

pub struct PaintCtx {
    pub transform: Affine,
    pub clip: Rect,
}

impl PaintCtx {
    pub fn paint_child(&mut self, child: &mut dyn Widget, scene: &mut Scene) {
        // In a real system, we would adjust transform/clip here based on layout
        child.paint(self, scene);
    }

    pub fn is_visible(&self, _rect: &Rect) -> bool {
        true
    }
}

pub enum Event {
    // Stub
}

pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

pub trait Widget {
    fn on_event(&mut self, _ctx: &mut EventCtx, _event: &Event) {}
    fn layout(&mut self, _ctx: &mut LayoutCtx, _bc: &BoxConstraints) -> Size {
        Size::ZERO
    }
    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene);
    fn children(&self) -> SmallVec<[EntityId; 4]> {
        SmallVec::new()
    }
}

pub trait AppExtension {
    fn mount_ui<F, E>(&mut self, builder: F)
    where
        F: FnMut() -> E + 'static,
        E: IntoElement;
}

impl AppExtension for felt_platform::App {
    fn mount_ui<F, E>(&mut self, mut builder: F)
    where
        F: FnMut() -> E + 'static,
        E: IntoElement,
    {
        self.mount(move |scene, width, height| {
            let mut root_widget = builder().into_element().build();

            let mut ctx = PaintCtx {
                transform: Affine::IDENTITY,
                clip: Rect::new(0.0, 0.0, width as f64, height as f64),
            };

            root_widget.paint(&mut ctx, scene);
        });
    }
}
