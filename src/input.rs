use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct InputState {
    left_pressed: bool,
    right_pressed: bool,
    forward_pressed: bool,
    backward_pressed: bool,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            left_pressed: false,
            right_pressed: false,
            forward_pressed: false,
            backward_pressed: false,
        }
    }

    pub fn apply_keyboard(&mut self, input: KeyboardInput) {
        match input {
            KeyboardInput {
                state,
                virtual_keycode,
                ..
            } => match virtual_keycode {
                Some(VirtualKeyCode::W) => self.forward_pressed = state == ElementState::Pressed,
                Some(VirtualKeyCode::A) => self.left_pressed = state == ElementState::Pressed,
                Some(VirtualKeyCode::S) => self.backward_pressed = state == ElementState::Pressed,
                Some(VirtualKeyCode::D) => self.right_pressed = state == ElementState::Pressed,
                _ => (),
            },
        }
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
}
