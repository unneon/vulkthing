use crate::interface::Interface;
use imgui::{Context, DrawData, FontSource};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::{CursorGrabMode, Window};

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
            WindowEvent::Focused(gained_focus) => {
                io.app_focus_lost = !*gained_focus;
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
            let _ = window.set_cursor_position(window_center);
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.cursor_visible = true;
        } else if !camera_lock && self.cursor_visible {
            let window_size = window.inner_size();
            let window_center = PhysicalPosition {
                x: window_size.width / 2,
                y: window_size.height / 2,
            };

            let _ = window.set_cursor_grab(CursorGrabMode::Locked);
            let _ = window.set_cursor_position(window_center);
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

    pub fn draw_data(&mut self) -> &DrawData {
        self.ctx.render()
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
