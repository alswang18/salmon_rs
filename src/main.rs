use anyhow::Result;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod software_canvas;
mod timing;
mod vec4;

use software_canvas::SoftwareCanvas;
use timing::FrameTiming;

struct App {
    /// The main window handle - wrapped in Arc for sharing with softbuffer
    window: Option<std::sync::Arc<Window>>,

    /// Fixed 64x64 canvas for all drawing operations with integrated graphics
    canvas: SoftwareCanvas,

    /// Frame timing and FPS management
    timing: FrameTiming,
}

const DEFAULT_SIZE: (u32, u32) = (640, 640);

impl ApplicationHandler for App {
    /// Called when the application is resumed or initially started.
    /// This is where we create the window and initialize graphics.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = std::sync::Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Salmon RS")
                        .with_inner_size(winit::dpi::LogicalSize::new(
                            DEFAULT_SIZE.0,
                            DEFAULT_SIZE.1,
                        ))
                        .with_resizable(true),
                )
                .expect("Failed to create window"),
        );

        self.canvas
            .initialize_graphics(window.clone())
            .expect("Failed to initialize graphics");

        self.window = Some(window.clone());

        // Request initial redraw
        if let Some(window) = &self.window {
            window.request_redraw();
        }
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
                if let Some(fps) = self.timing.update_fps() {
                    if let Some(window) = &self.window {
                        window.set_title(&format!("Salmon RS - FPS: {:.1}", fps));
                    }
                }
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
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Apply FPS limiting
        self.timing.apply_fps_limit();

        // Request redraw continuously for polling mode
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            canvas: SoftwareCanvas::new(64, 64),
            timing: FrameTiming::new(),
        }
    }

    fn draw(&mut self) {
        if let Some(window) = &self.window {
            let size = window.inner_size();
            if size.width == 0 || size.height == 0 {
                return;
            }

            // Ensure surface matches current window size
            if let Err(e) = self.canvas.ensure_surface_size(&window) {
                println!("Failed to ensure surface size: {}", e);
                return;
            }

            // Render the frame content
            self.canvas.render_frame();

            // Present the frame
            if let Err(e) = self.canvas.present_frame() {
                println!("Failed to present frame: {}", e);
            }
        }
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
