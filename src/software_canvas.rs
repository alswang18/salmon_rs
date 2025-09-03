use crate::vec4::Vec4;
use anyhow::Result;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::window::Window;

pub struct SoftwareCanvas {
    canvas_size: (u32, u32),
    window_size: (u32, u32),

    // Graphics infrastructure
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    context: Option<softbuffer::Context<Arc<Window>>>,
    current_surface_size: Option<(u32, u32)>,
}

impl SoftwareCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        let canvas_size = (width, height);
        Self {
            canvas_size,
            window_size: (0, 0),
            surface: None,
            context: None,
            current_surface_size: None,
        }
    }

    pub fn initialize_graphics(&mut self, window: Arc<Window>) -> Result<()> {
        let context = softbuffer::Context::new(window.clone()).expect("Failed to create context");
        let mut surface =
            softbuffer::Surface::new(&context, window.clone()).expect("Failed to create surface");

        // Initialize surface size immediately on macOS
        let size = window.inner_size();
        if size.width > 0 && size.height > 0 {
            surface
                .resize(
                    NonZeroU32::new(size.width).expect("Window width cannot be zero"),
                    NonZeroU32::new(size.height).expect("Window height cannot be zero"),
                )
                .expect("Failed to resize surface");
            self.current_surface_size = Some((size.width, size.height));
            self.window_size = (size.width, size.height);
        }

        self.context = Some(context);
        self.surface = Some(surface);
        Ok(())
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Vec4) {
        if x >= self.canvas_size.0 || y >= self.canvas_size.1 {
            return;
        }

        if let Some(surface) = &mut self.surface {
            if let Ok(mut buffer) = surface.buffer_mut() {
                let scale_x = self.window_size.0 as f32 / self.canvas_size.0 as f32;
                let scale_y = self.window_size.1 as f32 / self.canvas_size.1 as f32;

                // Calculate the range of surface pixels this canvas pixel covers
                let start_x = (x as f32 * scale_x) as u32;
                let end_x = ((x + 1) as f32 * scale_x) as u32;
                let start_y = (y as f32 * scale_y) as u32;
                let end_y = ((y + 1) as f32 * scale_y) as u32;

                let argb_color = color.to_argb();

                for surface_y in start_y..end_y.min(self.window_size.1) {
                    for surface_x in start_x..end_x.min(self.window_size.0) {
                        // Flip Y coordinate for surface
                        let real_y = self.window_size.1 - 1 - surface_y;
                        let index = (real_y * self.window_size.0 + surface_x) as usize;

                        if index < buffer.len() {
                            buffer[index] = argb_color;
                        }
                    }
                }
            }
        }
    }

    pub fn clear(&mut self, color: Vec4) {
        if let Some(surface) = &mut self.surface {
            if let Ok(mut buffer) = surface.buffer_mut() {
                let argb_color = color.to_argb();
                for pixel in buffer.iter_mut() {
                    *pixel = argb_color;
                }
            }
        }
    }

    pub fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Vec4) {
        // Simple Bresenham-like line drawing for testing debugging
        let steep = (x1 as i32 - x2 as i32).abs() < (y1 as i32 - y2 as i32).abs();
        let (mut start_x, mut start_y, mut end_x, mut end_y) = (x1, y1, x2, y2);
        if steep {
            std::mem::swap(&mut start_x, &mut start_y);
            std::mem::swap(&mut end_x, &mut end_y);
        }
        if start_x > end_x {
            std::mem::swap(&mut start_x, &mut end_x);
            std::mem::swap(&mut start_y, &mut end_y);
        }

        let dx = end_x - start_x;
        let dy = if end_y > start_y { end_y - start_y } else { start_y - end_y };
        
        for x in start_x..=end_x {
            let progress = if dx > 0 { (x - start_x) as f32 / dx as f32 } else { 0.0 };
            let y = if end_y > start_y {
                (start_y as f32 + dy as f32 * progress).round() as u32
            } else {
                (start_y as f32 - dy as f32 * progress).round() as u32
            };

            // Debug variables that should be visible in debugger
            let debug_x = x;
            let debug_y = y;
            let debug_progress = progress;
            let debug_color = color;
            
            println!("Drawing pixel at ({}, {}) with progress {:.2}", debug_x, debug_y, debug_progress);

            if steep {
                self.set_pixel(debug_y, debug_x, debug_color);
            } else {
                self.set_pixel(debug_x, debug_y, debug_color);
            }
        }
    }

    pub fn render_frame(&mut self) {
        // Clear with black background
        self.clear(Vec4::black());

        // All coordinates are in 64x64 canvas space
        let (ax, ay) = (7, 3);
        let (bx, by) = (12, 37);
        let (cx, cy) = (62, 53);

        self.draw_line(ax, ay, bx, by, Vec4::blue());
    }

    pub fn width(&self) -> u32 {
        self.canvas_size.0
    }

    pub fn height(&self) -> u32 {
        self.canvas_size.1
    }

    pub fn ensure_surface_size(&mut self, window: &Window) -> Result<()> {
        if let Some(surface) = &mut self.surface {
            let size = window.inner_size();
            let current_size = (size.width, size.height);

            // Always resize on macOS to ensure proper surface initialization
            if self.current_surface_size != Some(current_size) || cfg!(target_os = "macos") {
                if size.width > 0 && size.height > 0 {
                    surface
                        .resize(
                            NonZeroU32::new(size.width).expect("Window width cannot be zero"),
                            NonZeroU32::new(size.height).expect("Window height cannot be zero"),
                        )
                        .expect("Failed to resize surface");
                    self.current_surface_size = Some(current_size);
                    self.window_size = current_size;
                }
            }
        }
        Ok(())
    }

    pub fn present_frame(&mut self) -> Result<()> {
        if let Some(surface) = &mut self.surface {
            let buffer = surface.buffer_mut().expect("Failed to get surface buffer");
            buffer.present().expect("Failed to present buffer");
        }
        Ok(())
    }
}
