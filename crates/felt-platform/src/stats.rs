use crate::renderer::VSync;
use crate::simple_text::SimpleText;
use std::collections::VecDeque;
use vello::kurbo::{Affine, PathEl, Rect, Stroke};
use vello::peniko::color::palette;
use vello::peniko::{Brush, Color, Fill};
use vello::{AaConfig, Scene};

const SLIDING_WINDOW_SIZE: usize = 100;

#[derive(Debug)]
pub struct Snapshot {
    pub fps: f64,
    pub frame_time_ms: f64,
    pub frame_time_min_ms: f64,
    pub frame_time_max_ms: f64,
}

impl Snapshot {
    pub fn draw_layer(
        &self,
        scene: &mut Scene,
        text: &mut SimpleText,
        viewport_size: (f64, f64),
        samples: impl Iterator<Item = u64>,
        vsync: VSync,
        aa_config: AaConfig,
    ) {
        let (viewport_width, viewport_height) = viewport_size;
        let width = (viewport_width * 0.4).clamp(200., 600.);
        let height = width * 0.7;
        let x_offset = viewport_width - width;
        let y_offset = viewport_height - height;
        let offset = Affine::translate((x_offset, y_offset));

        // Draw the background
        scene.fill(
            Fill::NonZero,
            offset,
            palette::css::BLACK.with_alpha(0.75),
            None,
            &Rect::new(0., 0., width, height),
        );

        // Draw text labels
        let labels = [
            format!("Frame Time: {:.2} ms", self.frame_time_ms),
            format!("Frame Time (min): {:.2} ms", self.frame_time_min_ms),
            format!("Frame Time (max): {:.2} ms", self.frame_time_max_ms),
            format!(
                "VSync: {}",
                match vsync {
                    VSync::Off => "off",
                    VSync::On => "on",
                    VSync::Mailbox => "mailbox",
                }
            ),
            format!(
                "AA method: {}",
                match aa_config {
                    AaConfig::Area => "Analytic Area",
                    AaConfig::Msaa16 => "16xMSAA",
                    AaConfig::Msaa8 => "8xMSAA",
                }
            ),
            format!("Resolution: {viewport_width}x{viewport_height}"),
        ];

        let text_height = height * 0.5 / (1 + labels.len()) as f64;
        let left_margin = width * 0.01;
        let text_size = (text_height * 0.9) as f32;
        for (i, label) in labels.iter().enumerate() {
            text.add(
                scene,
                None,
                text_size,
                Some(&Brush::Solid(palette::css::WHITE)),
                offset * Affine::translate((left_margin, (i + 1) as f64 * text_height)),
                label,
            );
        }
        text.add(
            scene,
            None,
            text_size,
            Some(&Brush::Solid(palette::css::WHITE)),
            offset * Affine::translate((width * 0.67, text_height)),
            &format!("FPS: {:.2}", self.fps),
        );

        // Plot the samples with a bar graph
        use PathEl::*;
        let left_padding = width * 0.05;
        let graph_max_height = height * 0.5;
        let graph_max_width = width - 2. * (width * 0.01) - left_padding;
        let left_margin_padding = width * 0.01 + left_padding;
        let bar_extent = graph_max_width / (SLIDING_WINDOW_SIZE as f64);
        let bar_width = bar_extent * 0.4;
        let bar = [
            MoveTo((0., graph_max_height).into()),
            LineTo((0., 0.).into()),
            LineTo((bar_width, 0.).into()),
            LineTo((bar_width, graph_max_height).into()),
        ];

        let display_max = if self.frame_time_max_ms > 3. * self.frame_time_ms {
            round_up((1.33334 * self.frame_time_ms) as usize, 5) as f64
        } else {
            self.frame_time_max_ms
        };

        for (i, sample) in samples.enumerate() {
            let t = offset * Affine::translate((i as f64 * bar_extent, graph_max_height));
            let sample_ms = ((sample as f64) * 0.001).min(display_max);
            let h = sample_ms / display_max;
            let s = Affine::scale_non_uniform(1., -h);

            let color = match sample {
                ..=16_667 => Color::from_rgb8(100, 143, 255), // 60fps
                16_668..=33_334 => Color::from_rgb8(255, 176, 0), // 30fps
                _ => Color::from_rgb8(220, 38, 127),          // <30fps
            };

            scene.fill(
                Fill::NonZero,
                t * Affine::translate((
                    left_margin_padding,
                    (1 + labels.len()) as f64 * text_height,
                )) * s,
                color,
                None,
                &bar,
            );
        }

        // Draw threshold markers
        let marker = [
            MoveTo((0., graph_max_height).into()),
            LineTo((graph_max_width, graph_max_height).into()),
        ];

        let thresholds = [8.33, 16.66, 33.33];
        let thres_text_height = graph_max_height * 0.05;
        let thres_text_height_2 = thres_text_height * 0.5;
        for t in thresholds.iter().filter(|&&t| t < display_max) {
            let y = t / display_max;
            text.add(
                scene,
                None,
                thres_text_height as f32,
                Some(&Brush::Solid(palette::css::WHITE)),
                offset
                    * Affine::translate((
                        left_margin,
                        (2. - y) * graph_max_height + thres_text_height_2,
                    )),
                &format!("{t}"),
            );
            scene.stroke(
                &Stroke::new(graph_max_height * 0.01),
                offset * Affine::translate((left_margin_padding, (1. - y) * graph_max_height)),
                palette::css::WHITE,
                None,
                &marker,
            );
        }
    }
}

pub struct Sample {
    pub frame_time_us: u64,
}

pub struct Stats {
    count: usize,
    sum: u64,
    min: u64,
    max: u64,
    samples: VecDeque<u64>,
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

impl Stats {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            min: u64::MAX,
            max: u64::MIN,
            samples: VecDeque::with_capacity(SLIDING_WINDOW_SIZE),
        }
    }

    pub fn samples(&self) -> impl Iterator<Item = u64> + '_ {
        self.samples.iter().copied()
    }

    pub fn snapshot(&self) -> Snapshot {
        let frame_time_ms = (self.sum as f64 / self.count as f64) * 0.001;
        let fps = 1000. / frame_time_ms;
        Snapshot {
            fps,
            frame_time_ms,
            frame_time_min_ms: self.min as f64 * 0.001,
            frame_time_max_ms: self.max as f64 * 0.001,
        }
    }

    pub fn clear_min_and_max(&mut self) {
        self.min = u64::MAX;
        self.max = u64::MIN;
    }

    pub fn add_sample(&mut self, sample: Sample) {
        let oldest = if self.count < SLIDING_WINDOW_SIZE {
            self.count += 1;
            None
        } else {
            self.samples.pop_front()
        };

        let micros = sample.frame_time_us;
        self.sum += micros;
        self.samples.push_back(micros);

        if let Some(oldest) = oldest {
            self.sum -= oldest;
        }

        self.min = self.min.min(micros);
        self.max = self.max.max(micros);
    }
}

fn round_up(n: usize, f: usize) -> usize {
    n - 1 - (n - 1) % f + f
}
