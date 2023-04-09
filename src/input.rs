use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct InputState {
    left_pressed: bool,
    right_pressed: bool,
    forward_pressed: bool,
    backward_pressed: bool,
    jump: Click,
    sprint: bool,
    mouse_dx: f32,
    mouse_dy: f32,
}

#[derive(Default)]
struct Click {
    queued_count: usize,
    pressed: bool,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            left_pressed: false,
            right_pressed: false,
            forward_pressed: false,
            backward_pressed: false,
            jump: Click::default(),
            sprint: false,
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
            Some(VirtualKeyCode::Space) => self.jump.apply(input.state),
            Some(VirtualKeyCode::LShift) => self.sprint = input.state == ElementState::Pressed,
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
        self.jump.queued_count = 0;
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

    pub fn movement_jumps(&self) -> usize {
        self.jump.queued_count
    }

    pub fn movement_sprint(&self) -> bool {
        self.sprint
    }

    pub fn camera_yaw(&self) -> f32 {
        self.mouse_dx
    }

    pub fn camera_pitch(&self) -> f32 {
        self.mouse_dy
    }
}

impl Click {
    fn apply(&mut self, state: ElementState) {
        if state == ElementState::Pressed && !self.pressed {
            self.queued_count += 1;
            self.pressed = true;
        } else if state == ElementState::Released && self.pressed {
            self.pressed = false;
        }
    }
}
