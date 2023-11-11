use crate::cli::{Args, WindowProtocol};
use log::warn;
use winit::dpi::LogicalSize;
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::platform::wayland::EventLoopBuilderExtWayland;
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{CursorGrabMode, Fullscreen, WindowBuilder};

const TITLE: &str = "Vulkthing";
const INITIAL_SIZE: LogicalSize<f64> = LogicalSize::new(640., 360.);

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
}

pub fn create_window(args: &Args) -> Window {
    // Create the application window using winit. Use a predefined size for now, though games should
    // run in fullscreen eventually.
    let mut event_loop = EventLoopBuilder::new();
    match args.window_protocol {
        WindowProtocol::Wayland => event_loop.with_wayland(),
        WindowProtocol::X11 => event_loop.with_x11(),
    };
    let event_loop = event_loop.build().unwrap();
    let window = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(INITIAL_SIZE)
        .with_resizable(true)
        .with_decorations(false)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();
    if window.set_cursor_grab(CursorGrabMode::Locked).is_err() {
        warn!("cursor grab unavailable");
    }
    window.set_cursor_visible(false);
    Window { event_loop, window }
}
