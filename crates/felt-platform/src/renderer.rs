use std::sync::Arc;
use std::time::Instant;
use vello::util::{RenderContext, RenderSurface};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::Window;

use crate::simple_text::SimpleText;
use crate::stats::{Sample, Stats};

/// Controls vertical synchronization and frame presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VSync {
    /// No vsync. Presents frames immediately without waiting for display refresh.
    /// Lowest latency but may show tearing. Good for minimizing input lag.
    Off,

    /// Standard vsync. Waits for display refresh before presenting.
    /// Tear-free rendering but may stutter during irregular frame times.
    On,

    /// Triple buffering (mailbox mode). Renders without blocking but only presents
    /// the latest complete frame at vsync. Low latency + tear-free, but uses more VRAM.
    /// **Platform support**: Not supported on macOS (silently falls back to Fifo).
    Mailbox,
}

/// Configuration options for the renderer.
#[derive(Debug, Clone)]
pub struct RendererOptions {
    /// Display performance statistics overlay (FPS, frame times, etc).
    /// Stats appear in the bottom-right corner with a semi-transparent background.
    /// Can be toggled at runtime via `toggle_stats()` or `set_stats_shown()`.
    pub show_stats: bool,

    /// Controls vertical synchronization mode. On (default) for tear-free rendering,
    /// Off for lowest latency, Mailbox for both (requires more VRAM).
    /// Can be changed at runtime via `set_vsync()`.
    pub vsync: VSync,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            show_stats: true,
            vsync: VSync::On,
        }
    }
}

pub struct Renderer {
    context: RenderContext,
    surface: Option<RenderSurface<'static>>,
    vello_renderer: Option<vello::Renderer>,
    scale_factor: f64,
    stats: Stats,
    simple_text: SimpleText,
    show_stats: bool,
    vsync: VSync,
    last_frame_start: Option<Instant>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, options: RendererOptions) -> Result<Self, RendererError> {
        let mut context = RenderContext::new();

        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        let wgpu_surface = context.instance.create_surface(Arc::clone(&window))?;

        let present_mode = match options.vsync {
            VSync::Off => wgpu::PresentMode::Immediate,
            VSync::On => wgpu::PresentMode::Fifo,
            VSync::Mailbox => wgpu::PresentMode::Mailbox,
        };

        let mut surface = context
            .create_render_surface(
                wgpu_surface,
                size.width,
                size.height,
                present_mode,
            )
            .await?;

        // Override alpha mode to support transparency
        let dev_id = surface.dev_id;
        let caps = surface
            .surface
            .get_capabilities(context.devices[dev_id].adapter());
        let alpha_mode = caps
            .alpha_modes
            .iter()
            .find(|&&mode| mode == wgpu::CompositeAlphaMode::PreMultiplied)
            .or_else(|| {
                caps.alpha_modes
                    .iter()
                    .find(|&&mode| mode == wgpu::CompositeAlphaMode::PostMultiplied)
            })
            .copied()
            .unwrap_or(caps.alpha_modes[0]);

        surface.config.alpha_mode = alpha_mode;
        surface
            .surface
            .configure(&context.devices[dev_id].device, &surface.config);

        let device_handle = &context.devices[surface.dev_id];
        let vello_renderer =
            vello::Renderer::new(&device_handle.device, vello::RendererOptions::default())?;

        Ok(Self {
            context,
            surface: Some(surface),
            vello_renderer: Some(vello_renderer),
            scale_factor,
            stats: Stats::new(),
            simple_text: SimpleText::new(),
            show_stats: options.show_stats,
            vsync: options.vsync,
            last_frame_start: None,
        })
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn set_stats_shown(&mut self, shown: bool) {
        self.show_stats = shown;
    }

    pub fn stats_shown(&self) -> bool {
        self.show_stats
    }

    pub fn vsync(&self) -> VSync {
        self.vsync
    }

    /// Change vsync mode at runtime by reconfiguring the surface.
    /// Common use case: set to Immediate during resize for lowest latency,
    /// then restore to On/Mailbox when resize completes.
    pub fn set_vsync(&mut self, vsync: VSync) {
        self.vsync = vsync;

        if let Some(surface) = &mut self.surface {
            let present_mode = match vsync {
                VSync::Off => wgpu::PresentMode::Immediate,
                VSync::On => wgpu::PresentMode::Fifo,
                VSync::Mailbox => wgpu::PresentMode::Mailbox,
            };

            surface.config.present_mode = present_mode;
            surface
                .surface
                .configure(&self.context.devices[surface.dev_id].device, &surface.config);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0
            && height > 0
            && let Some(surface) = &mut self.surface
        {
            self.context.resize_surface(surface, width, height);
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    pub fn render(&mut self) -> Result<(), RendererError> {
        if self.show_stats {
            let frame_start = Instant::now();
            if let Some(last_start) = self.last_frame_start {
                let frame_time = frame_start.duration_since(last_start);
                self.stats.add_sample(Sample {
                    frame_time_us: frame_time.as_micros() as u64,
                });
            }
            self.last_frame_start = Some(frame_start);
        }

        let dev_id = self
            .surface
            .as_ref()
            .ok_or(RendererError::NoSurface)?
            .dev_id;
        let size = self.physical_size();

        let mut scene = vello::Scene::new();
        self.build_test_scene(&mut scene);

        if self.show_stats {
            let snapshot = self.stats.snapshot();
            snapshot.draw_layer(
                &mut scene,
                &mut self.simple_text,
                (size.width as f64, size.height as f64),
                self.stats.samples(),
                self.vsync,
                vello::AaConfig::Msaa16,
            );
        }

        let surface = self.surface.as_mut().unwrap();
        let renderer = self
            .vello_renderer
            .as_mut()
            .ok_or(RendererError::NoRenderer)?;
        let device_handle = &self.context.devices[dev_id];

        let surface_texture = surface.surface.get_current_texture()?;

        let render_params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width: size.width,
            height: size.height,
            antialiasing_method: vello::AaConfig::Msaa16,
        };

        renderer.render_to_texture(
            &device_handle.device,
            &device_handle.queue,
            &scene,
            &surface.target_view,
            &render_params,
        )?;

        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surface Blit"),
                });

        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        device_handle.queue.submit([encoder.finish()]);
        surface_texture.present();

        Ok(())
    }

    pub fn render_empty(&mut self) -> Result<(), RendererError> {
        let dev_id = self
            .surface
            .as_ref()
            .ok_or(RendererError::NoSurface)?
            .dev_id;
        let size = self.physical_size();

        let scene = vello::Scene::new();

        let surface = self.surface.as_mut().unwrap();
        let renderer = self
            .vello_renderer
            .as_mut()
            .ok_or(RendererError::NoRenderer)?;
        let device_handle = &self.context.devices[dev_id];

        let surface_texture = surface.surface.get_current_texture()?;

        let render_params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width: size.width,
            height: size.height,
            antialiasing_method: vello::AaConfig::Msaa16,
        };

        renderer.render_to_texture(
            &device_handle.device,
            &device_handle.queue,
            &scene,
            &surface.target_view,
            &render_params,
        )?;

        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surface Blit"),
                });

        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        device_handle.queue.submit([encoder.finish()]);
        surface_texture.present();

        Ok(())
    }

    pub fn render_benchmark(&mut self, rect_count: usize) -> Result<(), RendererError> {
        let dev_id = self
            .surface
            .as_ref()
            .ok_or(RendererError::NoSurface)?
            .dev_id;
        let size = self.physical_size();

        let mut scene = vello::Scene::new();
        self.build_benchmark_scene(&mut scene, rect_count);

        let surface = self.surface.as_mut().unwrap();
        let renderer = self
            .vello_renderer
            .as_mut()
            .ok_or(RendererError::NoRenderer)?;
        let device_handle = &self.context.devices[dev_id];

        let surface_texture = surface.surface.get_current_texture()?;

        let render_params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width: size.width,
            height: size.height,
            antialiasing_method: vello::AaConfig::Msaa16,
        };

        renderer.render_to_texture(
            &device_handle.device,
            &device_handle.queue,
            &scene,
            &surface.target_view,
            &render_params,
        )?;

        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surface Blit"),
                });

        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        device_handle.queue.submit([encoder.finish()]);
        surface_texture.present();

        Ok(())
    }

    fn build_test_scene(&self, scene: &mut vello::Scene) {
        use vello::kurbo::{Affine, Rect};
        use vello::peniko::Color;

        let Some(surface) = &self.surface else {
            return;
        };

        let width = surface.config.width as f64;
        let height = surface.config.height as f64;

        // Draw a red rectangle
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(255, 0, 0),
            None,
            &Rect::new(50.0, 50.0, 250.0, 150.0),
        );

        // Draw a green rectangle
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(0, 255, 0),
            None,
            &Rect::new(width - 250.0, 50.0, width - 50.0, 150.0),
        );

        // Draw a blue rectangle in the center
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(0, 0, 255),
            None,
            &Rect::new(
                center_x - 100.0,
                center_y - 100.0,
                center_x + 100.0,
                center_y + 100.0,
            ),
        );
    }

    pub fn build_benchmark_scene(&self, scene: &mut vello::Scene, count: usize) {
        use vello::kurbo::{Affine, Rect};
        use vello::peniko::Color;

        let Some(surface) = &self.surface else {
            return;
        };

        let width = surface.config.width as f64;
        let height = surface.config.height as f64;

        let cols = (count as f64).sqrt() as usize;
        let rows = count.div_ceil(cols);

        let rect_width = width / cols as f64;
        let rect_height = height / rows as f64;

        for i in 0..count {
            let col = i % cols;
            let row = i / cols;

            let x = col as f64 * rect_width;
            let y = row as f64 * rect_height;

            let color = Color::from_rgb8(
                ((i * 137) % 256) as u8,
                ((i * 211) % 256) as u8,
                ((i * 97) % 256) as u8,
            );

            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &Rect::new(x, y, x + rect_width, y + rect_height),
            );
        }
    }

    pub fn device(&self) -> Option<&wgpu::Device> {
        self.surface
            .as_ref()
            .map(|s| &self.context.devices[s.dev_id].device)
    }

    pub fn queue(&self) -> Option<&wgpu::Queue> {
        self.surface
            .as_ref()
            .map(|s| &self.context.devices[s.dev_id].queue)
    }

    pub fn config(&self) -> Option<&wgpu::SurfaceConfiguration> {
        self.surface.as_ref().map(|s| &s.config)
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn physical_size(&self) -> PhysicalSize<u32> {
        self.surface
            .as_ref()
            .map(|s| PhysicalSize::new(s.config.width, s.config.height))
            .unwrap_or(PhysicalSize::new(800, 600))
    }

    pub fn logical_size(&self) -> LogicalSize<f64> {
        self.physical_size().to_logical(self.scale_factor)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),

    #[error("Failed to request device: {0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),

    #[error("Failed to request adapter: {0}")]
    RequestAdapterError(#[from] wgpu::RequestAdapterError),

    #[error("Failed to create surface: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),

    #[error("Vello renderer error: {0}")]
    VelloError(#[from] vello::Error),

    #[error("No surface available")]
    NoSurface,

    #[error("No renderer available")]
    NoRenderer,
}
