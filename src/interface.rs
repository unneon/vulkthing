use crate::planet::Parameters;
use crate::renderer::uniform::Filters;
use crate::world::World;
use imgui::{Condition, Context, DrawData, FontSource, TreeNodeFlags, Ui};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, VirtualKeyCode, WindowEvent};
use winit::window::{CursorGrabMode, Window};

pub trait Editable {
    fn name(&self) -> &str;
    fn widget(&mut self, ui: &Ui) -> bool;
}

pub struct Interface {
    pub ctx: Context,
    cursor_visible: bool,
}

pub struct InterfaceEvents {
    pub planet_changed: bool,
}

impl Interface {
    pub fn new(width: usize, height: usize) -> Interface {
        let mut ctx = Context::create();
        ctx.set_ini_filename(None);
        ctx.fonts()
            .add_font(&[FontSource::DefaultFontData { config: None }]);
        ctx.io_mut().display_framebuffer_scale = [1., 1.];
        ctx.io_mut().display_size = [width as f32, height as f32];
        Interface {
            ctx,
            cursor_visible: false,
        }
    }

    pub fn apply_window(&mut self, event: &WindowEvent) {
        let io = self.ctx.io_mut();
        match event {
            WindowEvent::ReceivedCharacter(ch) => {
                if *ch != '\u{7f}' {
                    io.add_input_character(*ch);
                }
            }
            WindowEvent::Focused(gained_focus) => {
                io.app_focus_lost = !*gained_focus;
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    if let Some(key) = to_imgui_modifier(key) {
                        io.add_key_event(key, input.state == ElementState::Pressed);
                    }
                    if let Some(key) = to_imgui_key(key) {
                        io.add_key_event(key, input.state == ElementState::Pressed);
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                io.add_key_event(imgui::Key::ModShift, modifiers.shift());
                io.add_key_event(imgui::Key::ModCtrl, modifiers.ctrl());
                io.add_key_event(imgui::Key::ModAlt, modifiers.alt());
                io.add_key_event(imgui::Key::ModSuper, modifiers.logo());
            }
            WindowEvent::CursorMoved { position, .. } => {
                io.add_mouse_pos_event([position.x as f32, position.y as f32]);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mouse) = to_imgui_mouse(button) {
                    io.add_mouse_button_event(mouse, *state == ElementState::Pressed);
                }
            }
            WindowEvent::Resized(new_size) => {
                io.display_size = [new_size.width as f32, new_size.height as f32];
            }
            _ => (),
        }
    }

    pub fn apply_cursor(&mut self, camera_lock: bool, window: &Window) {
        if camera_lock && !self.cursor_visible {
            let window_size = window.inner_size();
            let window_center = PhysicalPosition {
                x: window_size.width / 2,
                y: window_size.height / 2,
            };
            window.set_cursor_position(window_center).unwrap();
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
            window.set_cursor_visible(true);
            self.cursor_visible = true;
        } else if !camera_lock && self.cursor_visible {
            let window_size = window.inner_size();
            let window_center = PhysicalPosition {
                x: window_size.width / 2,
                y: window_size.height / 2,
            };

            window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
            window.set_cursor_position(window_center).unwrap();
            window.set_cursor_visible(false);
            self.cursor_visible = false;
        }
        if self.ctx.io().want_set_mouse_pos {
            window
                .set_cursor_position(PhysicalPosition {
                    x: self.ctx.io().mouse_pos[0],
                    y: self.ctx.io().mouse_pos[1],
                })
                .unwrap();
        }
    }

    pub fn build(
        &mut self,
        world: &mut World,
        planet: &mut Parameters,
        filters: &mut Filters,
    ) -> InterfaceEvents {
        let mut planet_changed = false;
        let ui = self.ctx.frame();
        ui.window("Debugging")
            .size([0., 0.], Condition::Always)
            .build(|| {
                section(ui, world);
                planet_changed = section(ui, planet);
                section(ui, filters);
            });
        InterfaceEvents { planet_changed }
    }

    pub fn draw_data(&mut self) -> &DrawData {
        self.ctx.render()
    }
}

fn section(ui: &Ui, editable: &mut impl Editable) -> bool {
    ui.collapsing_header(editable.name(), TreeNodeFlags::empty())
        .then(|| editable.widget(ui))
        .unwrap_or(false)
}

fn to_imgui_modifier(key: VirtualKeyCode) -> Option<imgui::Key> {
    match key {
        VirtualKeyCode::LShift | VirtualKeyCode::RShift => Some(imgui::Key::ModShift),
        VirtualKeyCode::LControl | VirtualKeyCode::RControl => Some(imgui::Key::ModCtrl),
        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => Some(imgui::Key::ModAlt),
        VirtualKeyCode::LWin | VirtualKeyCode::RWin => Some(imgui::Key::ModSuper),
        _ => None,
    }
}

fn to_imgui_key(key: VirtualKeyCode) -> Option<imgui::Key> {
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

fn to_imgui_mouse(button: &MouseButton) -> Option<imgui::MouseButton> {
    // TODO: Convert Other button to Extra1/Extra2?
    match button {
        MouseButton::Left => Some(imgui::MouseButton::Left),
        MouseButton::Right => Some(imgui::MouseButton::Right),
        MouseButton::Middle => Some(imgui::MouseButton::Middle),
        _ => None,
    }
}
