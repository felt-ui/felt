use std::sync::Arc;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::Window;

#[allow(dead_code)]
pub struct Renderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scale_factor: f64,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Felt Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Prefer PreMultiplied alpha for transparency support
        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .find(|&&mode| mode == wgpu::CompositeAlphaMode::PreMultiplied)
            .or_else(|| {
                surface_caps
                    .alpha_modes
                    .iter()
                    .find(|&&mode| mode == wgpu::CompositeAlphaMode::PostMultiplied)
            })
            .copied()
            .unwrap_or(surface_caps.alpha_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // Vsync
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            scale_factor,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    pub fn render(&self) -> Result<(), RendererError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn physical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.config.width, self.config.height)
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
}
