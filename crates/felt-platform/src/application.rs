use crate::size::Size;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

type InitCallback = dyn for<'a> FnOnce(&mut AppContext<'a>);

#[derive(Default)]
pub struct Application {
    window: Option<Window>,
    init: Option<Box<InitCallback>>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            window: None,
            init: None,
        }
    }

    pub fn run<F>(mut self, init: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: for<'a> FnOnce(&mut AppContext<'a>) + 'static,
    {
        self.init = Some(Box::new(init));
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let mut cx = AppContext::new(event_loop);

            if let Some(init) = self.init.take() {
                init(&mut cx);
            }

            self.window = cx.window;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Render frame here
            }
            _ => {}
        }
    }
}

pub struct AppContext<'a> {
    event_loop: &'a ActiveEventLoop,
    window: Option<Window>,
}

impl<'a> AppContext<'a> {
    fn new(event_loop: &'a ActiveEventLoop) -> Self {
        Self {
            event_loop,
            window: None,
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

        let window = self.event_loop.create_window(attrs).unwrap();
        window.request_redraw();
        self.window = Some(window);
        self.window.as_ref().unwrap()
    }
}

pub struct WindowOptions {
    pub title: String,
    pub size: Size,
    pub window_decorations: Option<bool>,
    pub transparent: Option<bool>,
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
        assert!(app.init.is_none());
    }
}
