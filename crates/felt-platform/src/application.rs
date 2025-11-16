use crate::renderer::{Renderer, RendererOptions};
use crate::size::Size;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Controls when the window is redrawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedrawMode {
    /// Event-driven rendering. Only redraws when triggered by window events
    /// (resize, user input, explicit redraw requests). Energy efficient and
    /// recommended for most UI applications.
    #[default]
    OnDemand,

    /// Continuous rendering at maximum frame rate. Redraws every frame regardless
    /// of changes. Use for animations, live stats, benchmarks, or games where
    /// content updates constantly. Higher CPU/power usage.
    Continuous,
}

type InitCallback = dyn for<'a> FnOnce(&mut AppContext<'a>);

#[derive(Default)]
pub struct Application {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    init: Option<Box<InitCallback>>,
    redraw_mode: RedrawMode,
}

impl Application {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            init: None,
            redraw_mode: RedrawMode::default(),
        }
    }

    pub fn run<F>(mut self, init: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: for<'a> FnOnce(&mut AppContext<'a>) + 'static,
    {
        self.init = Some(Box::new(init));
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }

    pub fn redraw_mode(&self) -> RedrawMode {
        self.redraw_mode
    }

    /// Change redraw mode at runtime and update the event loop's ControlFlow.
    /// Common use case: switch to Continuous during animations, back to OnDemand when idle.
    pub fn set_redraw_mode(&mut self, event_loop: &ActiveEventLoop, redraw_mode: RedrawMode) {
        self.redraw_mode = redraw_mode;
        let control_flow = match redraw_mode {
            RedrawMode::OnDemand => ControlFlow::Wait,
            RedrawMode::Continuous => ControlFlow::Poll,
        };
        event_loop.set_control_flow(control_flow);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let mut cx = AppContext::new(event_loop);

            if let Some(init) = self.init.take() {
                init(&mut cx);
            }

            let redraw_mode = cx.redraw_mode();

            if let Some(window) = cx.window {
                let window = Arc::new(window);

                // Initialize renderer with the window
                match pollster::block_on(Renderer::new(
                    Arc::clone(&window),
                    RendererOptions::default(),
                )) {
                    Ok(renderer) => {
                        let control_flow = match redraw_mode {
                            RedrawMode::OnDemand => ControlFlow::Wait,
                            RedrawMode::Continuous => ControlFlow::Poll,
                        };
                        event_loop.set_control_flow(control_flow);
                        self.renderer = Some(renderer);
                        self.redraw_mode = redraw_mode;
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize renderer: {}", e);
                        event_loop.exit();
                        return;
                    }
                }

                window.request_redraw();
                self.window = Some(window);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer
                    && let Err(e) = renderer.render_empty()
                {
                    eprintln!("Render error: {}", e);
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                    if let Err(e) = renderer.render_empty() {
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

pub struct AppContext<'a> {
    event_loop: &'a ActiveEventLoop,
    window: Option<Window>,
    redraw_mode: RedrawMode,
}

impl<'a> AppContext<'a> {
    fn new(event_loop: &'a ActiveEventLoop) -> Self {
        Self {
            event_loop,
            window: None,
            redraw_mode: RedrawMode::default(),
        }
    }

    pub fn open_window(&mut self, options: WindowOptions) -> &Window {
        let mut attrs = Window::default_attributes()
            .with_title(options.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                options.size.width,
                options.size.height,
            ));

        if let Some(decorations) = options.window_decorations {
            attrs = attrs.with_decorations(decorations);
        }

        if let Some(transparent) = options.transparent {
            attrs = attrs.with_transparent(transparent);
        }

        self.redraw_mode = options.redraw_mode;

        let window = self.event_loop.create_window(attrs).unwrap();
        window.request_redraw();
        self.window = Some(window);
        self.window.as_ref().unwrap()
    }

    pub fn redraw_mode(&self) -> RedrawMode {
        self.redraw_mode
    }
}

pub struct WindowOptions {
    pub title: String,
    pub size: Size,
    pub window_decorations: Option<bool>,
    pub transparent: Option<bool>,
    pub redraw_mode: RedrawMode,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Felt UI".into(),
            size: Size {
                width: 800.0,
                height: 600.0,
            },
            window_decorations: None,
            transparent: None,
            redraw_mode: RedrawMode::OnDemand,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn creates_application_with_no_window() {
        let app = Application::new();
        assert!(app.window.is_none());
        assert!(app.renderer.is_none());
        assert!(app.init.is_none());
    }
}
