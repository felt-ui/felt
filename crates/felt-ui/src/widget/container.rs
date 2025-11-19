use crate::draw::Color;
use crate::{EntityId, PaintCtx, Widget};
use smallvec::SmallVec;
use vello::Scene;
use vello::kurbo::{Affine, Rect, Size, Stroke, Vec2};
use vello::peniko::{Brush, Fill};

pub struct Container {
    pub child: Option<Box<dyn Widget>>,
    pub background: Option<Color>,
    pub border: Option<(Color, f64)>, // Color, width
    pub offset: Vec2,
    pub size: Option<Size>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            child: None,
            background: None,
            border: None,
            offset: Vec2::ZERO,
            size: None,
        }
    }
}

impl Widget for Container {
    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        // Apply offset to transform
        let transform = ctx.transform.then_translate(self.offset);

        let mut my_ctx = PaintCtx {
            transform,
            clip: ctx.clip,
        };

        // Draw background if size is known
        if let Some(size) = self.size {
            let rect = Rect::from_origin_size(vello::kurbo::Point::ORIGIN, size);

            if let Some(color) = self.background {
                // Convert Felt UI Color to Vello Color
                let vello_color = vello::peniko::Color::rgba8(color.r, color.g, color.b, color.a);
                scene.fill(
                    Fill::NonZero,
                    transform,
                    &Brush::Solid(vello_color),
                    None,
                    &rect,
                );
            }

            if let Some((color, width)) = self.border {
                // Convert Felt UI Color to Vello Color
                let vello_color = vello::peniko::Color::rgba8(color.r, color.g, color.b, color.a);
                scene.stroke(
                    &Stroke::new(width),
                    transform,
                    &Brush::Solid(vello_color),
                    None,
                    &rect,
                );
            }
        } else if let Some(color) = self.background {
            // If no size but background, fill the whole clip (Window background case)
            let vello_color = vello::peniko::Color::rgba8(color.r, color.g, color.b, color.a);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(vello_color),
                None,
                &ctx.clip,
            );
        }

        if let Some(child) = &mut self.child {
            child.paint(&mut my_ctx, scene);
        }
    }

    fn children(&self) -> SmallVec<[EntityId; 4]> {
        SmallVec::new()
    }
}
