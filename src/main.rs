use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Debug, Clone, Copy)]
struct Vec4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Vec4 {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn pink() -> Self {
        Self::new(1.0, 0.0, 1.0, 1.0)
    }

    pub fn blue() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    pub fn green() -> Self {
        Self::new(0.0, 1.0, 0.0, 1.0)
    }

    pub fn yellow() -> Self {
        Self::new(1.0, 1.0, 0.0, 1.0)
    }

    pub fn red() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }

    fn to_argb(&self) -> u32 {
        let a = (self.a.clamp(0.0, 1.0) * 255.0) as u32;
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u32;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u32;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u32;

        (a << 24) | (r << 16) | (g << 8) | b
    }
}

impl std::fmt::Display for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Vec4(r: {:.3}, g: {:.3}, b: {:.3}, a: {:.3})",
            self.r, self.g, self.b, self.a
        )
    }
}

// Configuration constants
const ENABLE_FPS_LIMIT: bool = true;
const MAX_FPS: f32 = 240.0;

struct App {
    /// The main window handle - wrapped in Arc for sharing with softbuffer
    window: Option<std::sync::Arc<Window>>,

    /// Softbuffer surface - the drawable pixel buffer that represents the window's contents.
    /// This is where we write pixel data (ARGB values) that gets displayed on screen.
    /// Think of it as a 2D array of pixels that maps directly to what you see in the window.
    surface: Option<softbuffer::Surface<std::sync::Arc<Window>, std::sync::Arc<Window>>>,

    /// Softbuffer graphics context - manages the connection between the application and
    /// the underlying graphics system (X11, Wayland, etc.). It handles platform-specific
    /// details like memory allocation, pixel format conversion, and communicating with
    /// the window manager. Required to create surfaces.
    context: Option<softbuffer::Context<std::sync::Arc<Window>>>,

    /// Current surface size to avoid unnecessary resize operations
    current_size: Option<(u32, u32)>,

    // Timestamp of the last frame render - used for FPS limiting calculations
    last_frame_time: Instant,

    // Number of frames rendered since the last FPS display update
    frame_count_since_last_update: u32,

    // Timestamp of when the FPS counter was last updated in the window title
    fps_update_time: Instant,
}

const INTENDED_SIZE: (u32, u32) = (640, 640);

// We want to act as if this 640x640 window is actually 64x64 for rendering purposes
const BLOCK_SCALING: (u32, u32) = (10, 10);

impl ApplicationHandler for App {
    /// Called when the application is resumed or initially started.
    /// This is where we create the window, graphics context, and surface for rendering.
    /// On some platforms (like Android/iOS), this can be called multiple times if the app
    /// is suspended and resumed, but on desktop it's typically called once.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = std::sync::Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Salmon RS")
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            INTENDED_SIZE.0,
                            // The title bar is roughly 35 pixels tall
                            INTENDED_SIZE.1 + 35,
                        )),
                )
                .unwrap(),
        );

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        self.window = Some(window.clone());
        self.context = Some(context);
        self.surface = Some(surface);

        self.draw();
    }

    /// Handles all window-specific events like input, resizing, and close requests.
    /// This is the main event handler where we process user interactions and system events.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            // User clicked the window's close button (X) - exit the application
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // User pressed the Escape key - also exit the application
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            // Window needs to be redrawn - update FPS counter and render a new frame
            WindowEvent::RedrawRequested => {
                self.update_fps();
                self.draw();
            }
            // Window was resized by the user - trigger a redraw to fill the new size
            WindowEvent::Resized(_) => {
                self.draw();
            }
            // Ignore all other window events (mouse, keyboard, focus, etc.)
            _ => {}
        }
    }

    /// Called when the event loop is about to wait for new events.
    /// In polling mode, this is where we implement FPS limiting and request continuous redraws.
    /// This method is called after processing all pending events, right before the event loop
    /// would normally block waiting for new events to arrive.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Apply FPS limiting if enabled - sleep to maintain target framerate
        if ENABLE_FPS_LIMIT {
            let target_frame_duration = Duration::from_secs_f32(1.0 / MAX_FPS);
            let elapsed = self.last_frame_time.elapsed();

            if elapsed < target_frame_duration {
                let sleep_time = target_frame_duration - elapsed;
                std::thread::sleep(sleep_time);
            }
        }

        // Request redraw continuously for polling mode - keeps the render loop active
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            window: None,
            surface: None,
            context: None,
            current_size: None,
            last_frame_time: now,
            frame_count_since_last_update: 0,
            fps_update_time: now,
        }
    }

    fn update_fps(&mut self) {
        let now = Instant::now();
        self.frame_count_since_last_update += 1;

        // Update FPS every second
        if now.duration_since(self.fps_update_time) >= Duration::from_secs(1) {
            let fps = self.frame_count_since_last_update as f32
                / now.duration_since(self.fps_update_time).as_secs_f32();

            if let Some(window) = &self.window {
                window.set_title(&format!("Salmon RS - FPS: {:.1}", fps));
            }

            self.frame_count_since_last_update = 0;
            self.fps_update_time = now;
        }

        self.last_frame_time = now;
    }

    /// Ensures the surface is resized to match the current window size.
    /// Only performs the resize operation if the size has actually changed.
    /// Returns true if a resize was performed, false if no resize was needed.
    fn ensure_surface_size(&mut self) {
        if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
            let size = window.inner_size();
            let current_size = (size.width, size.height);

            if self.current_size != Some(current_size) {
                surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();
                self.current_size = Some(current_size);

                println!("Resized surface to: {}x{}", size.width, size.height);
                let inner_size = window.inner_size();
                println!(
                    "Window inner size: {}x{}",
                    inner_size.width, inner_size.height
                );

                // // For the purpose of this example, we need the inner size to be square.
                // debug_assert!(inner_size.width == inner_size.height);
            }
        }
    }

    fn set_block(&mut self, x: u32, y: u32, color: Vec4) {
        let (block_width, block_height) = BLOCK_SCALING;
        let x = x * block_width;
        let y = y * block_height;

        for dy in 0..block_height {
            for dx in 0..block_width {
                self.set_pixel(x + dx, y + dy, color);
            }
        }
    }

    fn set_block_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Vec4) {
        let step_size = 0.02_f32;

        let steps = (1.0 / step_size) as usize;

        for i in 0..steps {
            let t = i as f32 * step_size;
            let dx = (x2 as f32 - x1 as f32) * t;
            let dy = (y2 as f32 - y1 as f32) * t;
            let x = (x1 as f32 + dx).round() as i32;
            let y = (y1 as f32 + dy).round() as i32;
            self.set_block(x as u32, y as u32, color);
        }

        // translate the above cpp to rust
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Vec4) {
        if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
            let size = window.inner_size();

            // The y-axis is interpreted as 0 is down and height is up.
            // The default was the other way around.
            let real_y = size.height - 1 - y;
            if x >= size.width || real_y >= size.height {
                return;
            }

            let mut buffer = surface.buffer_mut().unwrap();
            let index = (real_y * size.width + x) as usize;
            if let Some(pixel) = buffer.get_mut(index) {
                *pixel = color.to_argb();
            }
        }
    }

    fn clear(&mut self, color: Vec4) {
        if let Some(surface) = &mut self.surface {
            let mut buffer = surface.buffer_mut().unwrap();

            // Fill with black background
            let argb = color.to_argb();
            for pixel in buffer.iter_mut() {
                *pixel = argb;
            }
        }
    }

    fn end_frame(&mut self) {
        if let Some(surface) = &mut self.surface {
            let buffer = surface.buffer_mut().unwrap();
            buffer.present().unwrap();
        }
    }

    fn draw(&mut self) {
        // Ensure surface matches current window size
        self.ensure_surface_size();

        // Inner size is the actual drawable area.
        if let Some(window) = &self.window {
            let size = window.inner_size();
            if size.width == 0 || size.height == 0 {
                return;
            }
            size
        } else {
            return;
        };

        self.clear(Vec4::black());

        // All of these coordinates are in the block scaled version of screen space. (64 x 64)
        let (ax, ay) = (7, 3);
        let (bx, by) = (12, 37);
        let (cx, cy) = (62, 53);

        self.set_block_line(ax, ay, bx, by, Vec4::blue());
        self.set_block_line(cx, cy, bx, by, Vec4::green());
        self.set_block_line(cx, cy, ax, ay, Vec4::yellow());
        self.set_block_line(ax, ay, cx, cy, Vec4::red());

        self.end_frame();
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
