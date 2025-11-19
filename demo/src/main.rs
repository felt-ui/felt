use felt_platform::App;
use felt_ui::{canvas, div, scroll_view, AppExtension, IntoElement, PaintCtx, Widget};
use std::time::Instant;
use vello::kurbo::{Affine, Point, Rect, Size, Vec2};
use vello::peniko::{Brush, Color, Fill, Mix};
use vello::Scene;

fn main() {
    env_logger::init();

    let mut app = App::new();

    let start_time = Instant::now();

    println!("Running PoC V4 - 3 Distinct Layers");

    app.mount_ui(move || {
        // For every frame, transfirm scroll offset automatically
        // In the future we will use gesture events to control this
        let t = start_time.elapsed().as_secs_f64();
        let scroll_offset_y = (t * 1.0).sin() * 400.0 + 400.0;

        // Rebuild the widget tree every frame (Declarative Style!)
        // Now the entire scene is described as a widget tree.
        div()
            .bg(Color::rgb8(10, 10, 10)) // Window Background
            .child(
                div() // Container
                    .offset(Vec2::new(100.0, 100.0))
                    .size(Size::new(600.0, 400.0))
                    .bg(Color::rgb8(40, 40, 40)) // Container Background
                    .border(Color::rgb8(150, 150, 150), 4.0) // Container Border
                    .child(
                        scroll_view()
                            .size(Vec2::new(600.0, 400.0))
                            .offset(Vec2::new(0.0, scroll_offset_y))
                            .child(
                                div() // LAYER 2: SCROLL PANEL (The moving surface)
                                    .size(Size::new(500.0, 1200.0))
                                    .offset(Vec2::new(50.0, 0.0)) // Centered in 600px width
                                    .bg(Color::rgb8(80, 80, 80)) // Medium Gray Panel
                                    .child(
                                        div() // Wrapper for positioning the canvas
                                            .offset(Vec2::new(50.0, 50.0))
                                            .bg(Color::rgb8(60, 60, 100)) // Blue-ish Canvas Background
                                            .child(
                                                canvas(move |ctx, scene| {
                                                    // Draw diagonal stripes
                                                    for i in 0..22 {
                                                        let y = i as f64 * 50.0;
                                                        scene.fill(
                                                            Fill::NonZero,
                                                            ctx.transform,
                                                            &Brush::Solid(Color::rgb8(70, 70, 110)),
                                                            None,
                                                            &Rect::new(0.0, y, 400.0, y + 25.0),
                                                        );
                                                    }

                                                    // Header
                                                    scene.fill(
                                                        Fill::NonZero,
                                                        ctx.transform,
                                                        &Brush::Solid(Color::rgb8(200, 50, 50)),
                                                        None,
                                                        &Rect::new(0.0, 0.0, 400.0, 50.0),
                                                    );

                                                    // Footer
                                                    scene.fill(
                                                        Fill::NonZero,
                                                        ctx.transform,
                                                        &Brush::Solid(Color::rgb8(50, 200, 50)),
                                                        None,
                                                        &Rect::new(0.0, 1050.0, 400.0, 1100.0),
                                                    );

                                                    // CLIP CONTENT TO CANVAS RECT
                                                    let canvas_bounds = Rect::new(0.0, 0.0, 400.0, 1100.0);
                                                    let global_canvas_clip = ctx.transform.transform_rect_bbox(canvas_bounds);
                                                    scene.push_layer(Mix::Normal, 1.0, Affine::IDENTITY, &global_canvas_clip);

                                                    // Circles
                                                    for i in 0..50 {
                                                        let y = 50.0 + i as f64 * 40.0;
                                                        let x = 200.0 + (t * 2.0 + i as f64 * 0.2).sin() * 250.0;
                                                        scene.fill(
                                                            Fill::NonZero,
                                                            ctx.transform,
                                                            &Brush::Solid(Color::rgb8(200, 200, 255)),
                                                            None,
                                                            &vello::kurbo::Circle::new(Point::new(x, y), 15.0),
                                                        );
                                                    }
                                                    scene.pop_layer();
                                                })
                                                .size(Size::new(400.0, 1100.0))
                                            )
                                    )
                                    // We need to wrap the canvas in a div to offset it to (50, 50)
                                    // But `div().child(canvas)` works.
                                )
                            )
                    )
                });

    app.run();
}
