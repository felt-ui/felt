use felt_platform::renderer::{Renderer, RendererOptions};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const RECT_COUNT: usize = 200_000;

struct BenchmarkApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_times: Vec<Duration>,
    total_frames: usize,
}

impl BenchmarkApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            frame_times: Vec::new(),
            total_frames: 0,
        }
    }
}

impl ApplicationHandler for BenchmarkApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(format!("Vello Benchmark - {} rectangles", RECT_COUNT))
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
                .with_transparent(false)
                .with_resizable(false);

            let window = Arc::new(event_loop.create_window(attrs).unwrap());

            match pollster::block_on(Renderer::new(Arc::clone(&window), RendererOptions::default())) {
                Ok(renderer) => {
                    event_loop.set_control_flow(ControlFlow::Poll);
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
                if !self.frame_times.is_empty() {
                    let avg_frame_time: Duration = self
                        .frame_times
                        .iter()
                        .sum::<Duration>()
                        .div_f64(self.frame_times.len() as f64);

                    let fps = 1.0 / avg_frame_time.as_secs_f64();

                    println!("\nBenchmark Results ({} rectangles):", RECT_COUNT);
                    println!("  Total frames: {}", self.total_frames);
                    println!(
                        "  Average frame time: {:.2}ms",
                        avg_frame_time.as_secs_f64() * 1000.0
                    );
                    println!("  Average FPS: {:.2}", fps);
                    println!(
                        "  Min frame time: {:.2}ms",
                        self.frame_times.iter().min().unwrap().as_secs_f64() * 1000.0
                    );
                    println!(
                        "  Max frame time: {:.2}ms",
                        self.frame_times.iter().max().unwrap().as_secs_f64() * 1000.0
                    );
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    let frame_start = Instant::now();

                    if let Err(e) = renderer.render_benchmark(RECT_COUNT) {
                        eprintln!("Render error: {}", e);
                    }

                    let frame_time = frame_start.elapsed();
                    self.frame_times.push(frame_time);
                    self.total_frames += 1;

                    if self.total_frames % 60 == 0 {
                        let recent_avg: Duration = self
                            .frame_times
                            .iter()
                            .rev()
                            .take(60)
                            .sum::<Duration>()
                            .div_f64(60.0);
                        let fps = 1.0 / recent_avg.as_secs_f64();
                        println!(
                            "Frame {}: {:.2}ms ({:.1} FPS avg over last 60 frames)",
                            self.total_frames,
                            recent_avg.as_secs_f64() * 1000.0,
                            fps
                        );
                    }

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                    if let Err(e) = renderer.render_benchmark(RECT_COUNT) {
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
    println!("Starting Vello benchmark with {} rectangles...", RECT_COUNT);
    println!("Close the window to see final results.\n");

    let event_loop = EventLoop::new()?;

    let mut app = BenchmarkApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
