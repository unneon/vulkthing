use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct InputState {
    left_pressed: bool,
    right_pressed: bool,
    forward_pressed: bool,
    backward_pressed: bool,
    mouse_dx: f32,
    mouse_dy: f32,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            left_pressed: false,
            right_pressed: false,
            forward_pressed: false,
            backward_pressed: false,
            mouse_dx: 0.,
            mouse_dy: 0.,
        }
    }

    pub fn apply_keyboard(&mut self, input: KeyboardInput) {
        match input.virtual_keycode {
            Some(VirtualKeyCode::W) => self.forward_pressed = input.state == ElementState::Pressed,
            Some(VirtualKeyCode::A) => self.left_pressed = input.state == ElementState::Pressed,
            Some(VirtualKeyCode::S) => self.backward_pressed = input.state == ElementState::Pressed,
            Some(VirtualKeyCode::D) => self.right_pressed = input.state == ElementState::Pressed,
            _ => (),
        }
    }

    pub fn apply_mouse(&mut self, delta: (f64, f64)) {
        self.mouse_dx = delta.0 as f32;
        self.mouse_dy = delta.1 as f32;
    }

    pub fn reset_after_frame(&mut self) {
        self.mouse_dx = 0.;
        self.mouse_dy = 0.;
    }

    pub fn movement_horizontal(&self) -> f32 {
        let mut sum = 0.;
        if self.left_pressed {
            sum -= 1.;
        }
        if self.right_pressed {
            sum += 1.;
        }
        sum
    }

    pub fn movement_depth(&self) -> f32 {
        let mut sum = 0.;
        if self.backward_pressed {
            sum -= 1.;
        }
        if self.forward_pressed {
            sum += 1.;
        }
        sum
    }

    pub fn camera_yaw(&self) -> f32 {
        self.mouse_dx
    }
}
