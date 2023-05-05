use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, Fullscreen, WindowBuilder};

const TITLE: &str = "Vulkthing";
const INITIAL_SIZE: (usize, usize) = (640, 360);

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
}

pub fn create_window() -> Window {
    // Create the application window using winit. Use a predefined size for now, though games should
    // run in fullscreen eventually.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(
            INITIAL_SIZE.0 as f64,
            INITIAL_SIZE.1 as f64,
        ))
        .with_resizable(true)
        .with_decorations(false)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();
    window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
    window.set_cursor_visible(false);
    Window { event_loop, window }
}
