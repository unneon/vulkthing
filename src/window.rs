use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, Fullscreen, WindowBuilder};

const TITLE: &str = "Vulkthing";
const INITIAL_SIZE: LogicalSize<f64> = LogicalSize::new(640., 360.);
const DEMO_SIZE: LogicalSize<f64> = LogicalSize::new(640., 360.);

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
}

pub fn create_window(demo: bool) -> Window {
    // Create the application window using winit. Use a predefined size for now, though games should
    // run in fullscreen eventually.
    let event_loop = EventLoop::new();
    let inner_size = if demo { DEMO_SIZE } else { INITIAL_SIZE };
    let fullscreen = if demo {
        None
    } else {
        Some(Fullscreen::Borderless(None))
    };
    let window = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(inner_size)
        .with_resizable(true)
        .with_decorations(false)
        .with_fullscreen(fullscreen)
        .build(&event_loop)
        .unwrap();
    if !demo {
        window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
        window.set_cursor_visible(false);
    }
    Window { event_loop, window }
}
