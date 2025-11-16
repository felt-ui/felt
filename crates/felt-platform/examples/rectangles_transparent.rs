use felt_platform::renderer::{Renderer, RendererOptions};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct RectanglesApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl RectanglesApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
        }
    }
}

impl ApplicationHandler for RectanglesApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Rectangles Demo - Transparent")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
                .with_transparent(true);

            let window = Arc::new(event_loop.create_window(attrs).unwrap());

            match pollster::block_on(Renderer::new(Arc::clone(&window), RendererOptions::default())) {
                Ok(renderer) => {
                    event_loop.set_control_flow(ControlFlow::Wait);
                    self.renderer = Some(renderer);
                    window.request_redraw();
                }
                Err(e) => {
                    eprintln!("Failed to initialize renderer: {}", e);
                    event_loop.exit();
                    return;
                }
            }

            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render() {
                        eprintln!("Render error: {}", e);
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                    if let Err(e) = renderer.render() {
                        eprintln!("Render error during resize: {}", e);
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.set_scale_factor(scale_factor);
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let mut app = RectanglesApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
