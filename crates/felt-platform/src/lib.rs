use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{Renderer, Scene};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

pub struct App {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    surface: Option<RenderSurface<'static>>,
    window: Option<Arc<Window>>,
    paint_callback: Option<Box<dyn FnMut(&mut vello::Scene, u32, u32)>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            surface: None,
            window: None,
            paint_callback: None,
        }
    }

    pub fn mount(&mut self, callback: impl FnMut(&mut vello::Scene, u32, u32) + 'static) {
        self.paint_callback = Some(Box::new(callback));
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .build(&event_loop)
                .unwrap(),
        );
        self.window = Some(window.clone());

        let surface = pollster::block_on(self.context.create_surface(
            window.clone(),
            window.inner_size().width,
            window.inner_size().height,
            vello::wgpu::PresentMode::AutoVsync,
        ))
        .unwrap();

        self.surface = Some(surface);
        self.renderers.resize_with(1, || None);

        event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => elwt.exit(),
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        ..
                    } => {
                        if let Some(surface) = &mut self.surface {
                            self.context
                                .resize_surface(surface, size.width, size.height);
                        }
                        self.window.as_ref().unwrap().request_redraw();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        if let (Some(surface), Some(window)) = (&mut self.surface, &self.window) {
                            let width = surface.config.width;
                            let height = surface.config.height;
                            let device_handle = &self.context.devices[surface.dev_id];

                            let surface_texture = surface.surface.get_current_texture().unwrap();

                            let renderer =
                                self.renderers[surface.dev_id].get_or_insert_with(|| {
                                    Renderer::new(
                                        &device_handle.device,
                                        vello::RendererOptions {
                                            surface_format: Some(surface.format),
                                            use_cpu: false,
                                            antialiasing_support: vello::AaSupport::all(),
                                            num_init_threads: None,
                                        },
                                    )
                                    .unwrap()
                                });

                            let mut scene = vello::Scene::new();

                            // Call the paint callback
                            if let Some(callback) = &mut self.paint_callback {
                                callback(&mut scene, width, height);
                            } else {
                                // Default clear
                                scene.fill(
                                    vello::peniko::Fill::NonZero,
                                    vello::kurbo::Affine::IDENTITY,
                                    &vello::peniko::Brush::Solid(vello::peniko::Color::BLACK),
                                    None,
                                    &vello::kurbo::Rect::new(0.0, 0.0, width as f64, height as f64),
                                );
                            }

                            renderer
                                .render_to_surface(
                                    &device_handle.device,
                                    &device_handle.queue,
                                    &scene,
                                    &surface_texture,
                                    &vello::RenderParams {
                                        base_color: vello::peniko::Color::BLACK,
                                        width,
                                        height,
                                        antialiasing_method: vello::AaConfig::Area,
                                    },
                                )
                                .unwrap();

                            surface_texture.present();

                            // Request next frame for animation
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }
            })
            .unwrap();
    }
}
