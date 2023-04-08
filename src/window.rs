use winit::event_loop::EventLoop;
use winit::window::{Window as WinitWindow, WindowBuilder};

const TITLE: &str = "Vulkthing";
const INITIAL_SIZE: (usize, usize) = (1920, 1080);

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: WinitWindow,
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
        .with_resizable(false)
        .with_decorations(false)
        .build(&event_loop)
        .unwrap();
    Window { event_loop, window }
}
