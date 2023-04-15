use winit::event::{MouseButton, VirtualKeyCode};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, WindowBuilder};

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
    // window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
    // window.set_cursor_visible(false);
    Window { event_loop, window }
}

pub fn to_imgui_modifier(key: VirtualKeyCode) -> Option<imgui::Key> {
    match key {
        VirtualKeyCode::LShift | VirtualKeyCode::RShift => Some(imgui::Key::ModShift),
        VirtualKeyCode::LControl | VirtualKeyCode::RControl => Some(imgui::Key::ModCtrl),
        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => Some(imgui::Key::ModAlt),
        VirtualKeyCode::LWin | VirtualKeyCode::RWin => Some(imgui::Key::ModSuper),
        _ => None,
    }
}

pub fn to_imgui_key(key: VirtualKeyCode) -> Option<imgui::Key> {
    match key {
        VirtualKeyCode::Tab => Some(imgui::Key::Tab),
        VirtualKeyCode::Left => Some(imgui::Key::LeftArrow),
        VirtualKeyCode::Right => Some(imgui::Key::RightArrow),
        VirtualKeyCode::Up => Some(imgui::Key::UpArrow),
        VirtualKeyCode::Down => Some(imgui::Key::DownArrow),
        VirtualKeyCode::PageUp => Some(imgui::Key::PageUp),
        VirtualKeyCode::PageDown => Some(imgui::Key::PageDown),
        VirtualKeyCode::Home => Some(imgui::Key::Home),
        VirtualKeyCode::End => Some(imgui::Key::End),
        VirtualKeyCode::Insert => Some(imgui::Key::Insert),
        VirtualKeyCode::Delete => Some(imgui::Key::Delete),
        VirtualKeyCode::Back => Some(imgui::Key::Backspace),
        VirtualKeyCode::Space => Some(imgui::Key::Space),
        VirtualKeyCode::Return => Some(imgui::Key::Enter),
        VirtualKeyCode::Escape => Some(imgui::Key::Escape),
        VirtualKeyCode::LControl => Some(imgui::Key::LeftCtrl),
        VirtualKeyCode::LShift => Some(imgui::Key::LeftShift),
        VirtualKeyCode::LAlt => Some(imgui::Key::LeftAlt),
        VirtualKeyCode::LWin => Some(imgui::Key::LeftSuper),
        VirtualKeyCode::RControl => Some(imgui::Key::RightCtrl),
        VirtualKeyCode::RShift => Some(imgui::Key::RightShift),
        VirtualKeyCode::RAlt => Some(imgui::Key::RightAlt),
        VirtualKeyCode::RWin => Some(imgui::Key::RightSuper),
        // TODO: Menu key?
        VirtualKeyCode::Key0 => Some(imgui::Key::Alpha0),
        VirtualKeyCode::Key1 => Some(imgui::Key::Alpha1),
        VirtualKeyCode::Key2 => Some(imgui::Key::Alpha2),
        VirtualKeyCode::Key3 => Some(imgui::Key::Alpha3),
        VirtualKeyCode::Key4 => Some(imgui::Key::Alpha4),
        VirtualKeyCode::Key5 => Some(imgui::Key::Alpha5),
        VirtualKeyCode::Key6 => Some(imgui::Key::Alpha6),
        VirtualKeyCode::Key7 => Some(imgui::Key::Alpha7),
        VirtualKeyCode::Key8 => Some(imgui::Key::Alpha8),
        VirtualKeyCode::Key9 => Some(imgui::Key::Alpha9),
        VirtualKeyCode::A => Some(imgui::Key::A),
        VirtualKeyCode::B => Some(imgui::Key::B),
        VirtualKeyCode::C => Some(imgui::Key::C),
        VirtualKeyCode::D => Some(imgui::Key::D),
        VirtualKeyCode::E => Some(imgui::Key::E),
        VirtualKeyCode::F => Some(imgui::Key::F),
        VirtualKeyCode::G => Some(imgui::Key::G),
        VirtualKeyCode::H => Some(imgui::Key::H),
        VirtualKeyCode::I => Some(imgui::Key::I),
        VirtualKeyCode::J => Some(imgui::Key::J),
        VirtualKeyCode::K => Some(imgui::Key::K),
        VirtualKeyCode::L => Some(imgui::Key::L),
        VirtualKeyCode::M => Some(imgui::Key::M),
        VirtualKeyCode::N => Some(imgui::Key::N),
        VirtualKeyCode::O => Some(imgui::Key::O),
        VirtualKeyCode::P => Some(imgui::Key::P),
        VirtualKeyCode::Q => Some(imgui::Key::Q),
        VirtualKeyCode::R => Some(imgui::Key::R),
        VirtualKeyCode::S => Some(imgui::Key::S),
        VirtualKeyCode::T => Some(imgui::Key::T),
        VirtualKeyCode::U => Some(imgui::Key::U),
        VirtualKeyCode::V => Some(imgui::Key::V),
        VirtualKeyCode::W => Some(imgui::Key::W),
        VirtualKeyCode::X => Some(imgui::Key::X),
        VirtualKeyCode::Y => Some(imgui::Key::Y),
        VirtualKeyCode::Z => Some(imgui::Key::Z),
        VirtualKeyCode::F1 => Some(imgui::Key::F1),
        VirtualKeyCode::F2 => Some(imgui::Key::F2),
        VirtualKeyCode::F3 => Some(imgui::Key::F3),
        VirtualKeyCode::F4 => Some(imgui::Key::F4),
        VirtualKeyCode::F5 => Some(imgui::Key::F5),
        VirtualKeyCode::F6 => Some(imgui::Key::F6),
        VirtualKeyCode::F7 => Some(imgui::Key::F7),
        VirtualKeyCode::F8 => Some(imgui::Key::F8),
        VirtualKeyCode::F9 => Some(imgui::Key::F9),
        VirtualKeyCode::F10 => Some(imgui::Key::F10),
        VirtualKeyCode::F11 => Some(imgui::Key::F11),
        VirtualKeyCode::F12 => Some(imgui::Key::F12),
        VirtualKeyCode::Apostrophe => Some(imgui::Key::Apostrophe),
        VirtualKeyCode::Comma => Some(imgui::Key::Comma),
        VirtualKeyCode::Minus => Some(imgui::Key::Minus),
        VirtualKeyCode::Period => Some(imgui::Key::Period),
        VirtualKeyCode::Slash => Some(imgui::Key::Slash),
        VirtualKeyCode::Semicolon => Some(imgui::Key::Semicolon),
        VirtualKeyCode::Equals => Some(imgui::Key::Equal),
        VirtualKeyCode::LBracket => Some(imgui::Key::LeftBracket),
        VirtualKeyCode::Backslash => Some(imgui::Key::Backslash),
        VirtualKeyCode::RBracket => Some(imgui::Key::RightBracket),
        VirtualKeyCode::Grave => Some(imgui::Key::GraveAccent),
        // TODO: Caps lock?
        VirtualKeyCode::Scroll => Some(imgui::Key::ScrollLock),
        VirtualKeyCode::Numlock => Some(imgui::Key::NumLock),
        VirtualKeyCode::Snapshot => Some(imgui::Key::PrintScreen),
        VirtualKeyCode::Pause => Some(imgui::Key::Pause),
        VirtualKeyCode::Numpad0 => Some(imgui::Key::Keypad0),
        VirtualKeyCode::Numpad1 => Some(imgui::Key::Keypad1),
        VirtualKeyCode::Numpad2 => Some(imgui::Key::Keypad2),
        VirtualKeyCode::Numpad3 => Some(imgui::Key::Keypad3),
        VirtualKeyCode::Numpad4 => Some(imgui::Key::Keypad4),
        VirtualKeyCode::Numpad5 => Some(imgui::Key::Keypad5),
        VirtualKeyCode::Numpad6 => Some(imgui::Key::Keypad6),
        VirtualKeyCode::Numpad7 => Some(imgui::Key::Keypad7),
        VirtualKeyCode::Numpad8 => Some(imgui::Key::Keypad8),
        VirtualKeyCode::Numpad9 => Some(imgui::Key::Keypad9),
        VirtualKeyCode::NumpadDecimal => Some(imgui::Key::KeypadDecimal),
        VirtualKeyCode::NumpadDivide => Some(imgui::Key::KeypadDivide),
        VirtualKeyCode::NumpadMultiply => Some(imgui::Key::KeypadMultiply),
        VirtualKeyCode::NumpadSubtract => Some(imgui::Key::KeypadSubtract),
        VirtualKeyCode::NumpadAdd => Some(imgui::Key::KeypadAdd),
        VirtualKeyCode::NumpadEnter => Some(imgui::Key::KeypadEnter),
        VirtualKeyCode::NumpadEquals => Some(imgui::Key::KeypadEqual),
        // TODO: Gamepad keys?
        // TODO: Mouse buttons as keys?
        // TODO: Mouse wheel as key?
        // TODO: Reserved modifier key codes?
        // TODO: Shortcut modifier?
        _ => None,
    }
}

pub fn to_imgui_mouse(button: MouseButton) -> Option<imgui::MouseButton> {
    // TODO: Convert Other button to Extra1/Extra2?
    match button {
        MouseButton::Left => Some(imgui::MouseButton::Left),
        MouseButton::Right => Some(imgui::MouseButton::Right),
        MouseButton::Middle => Some(imgui::MouseButton::Middle),
        _ => None,
    }
}
