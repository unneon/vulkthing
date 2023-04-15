#![feature(array_chunks)]
#![feature(const_cstr_methods)]
#![feature(const_option)]
#![feature(const_result_drop)]
#![feature(generic_const_exprs)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_write_slice)]
#![allow(incomplete_features)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]
#![allow(clippy::too_many_arguments)]

mod camera;
mod input;
mod logger;
mod model;
mod renderer;
mod window;
mod world;

use crate::input::InputState;
use crate::logger::initialize_logger;
use crate::model::load_model;
use crate::renderer::Renderer;
use crate::window::create_window;
use crate::world::World;
use std::time::Instant;
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, ElementState, Event, MouseButton, StartCause, VirtualKeyCode, WindowEvent,
};

const VULKAN_APP_NAME: &str = "Vulkthing";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "Unneongine";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

const WALK_SPEED: f32 = 1.5;
const SPRINT_SPEED: f32 = 5.;
const CAMERA_SENSITIVITY: f32 = 0.01;

fn main() {
    initialize_logger();
    let window = create_window();
    let cube_model = load_model("assets/cube.obj", "assets/cube.png");
    let building_model = load_model("assets/czudec-pkp.obj", "assets/czudec-pkp.jpg");
    let mut renderer = Renderer::new(&window, &[building_model, cube_model]);
    let mut input_state = InputState::new();
    let mut world = World::new();
    let mut last_update = Instant::now();

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, always rendering after
    // MainEventsCleared is enough.
    window.event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => (),
            // Can be used for collecting frame timing information later. Specifically, this makes
            // it possible to measure frame times accounting for things like having multiple input
            // events before a redraw request.
            Event::NewEvents(StartCause::Poll) => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::ReceivedCharacter(ch) => {
                    if ch != '\u{7f}' {
                        renderer.imgui.io_mut().add_input_character(ch);
                    }
                }
                WindowEvent::Focused(gained_focus) => {
                    renderer.imgui.io_mut().app_focus_lost = !gained_focus;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    input_state.apply_keyboard(input);
                    if let Some(key) = input.virtual_keycode {
                        if let Some(key) = to_imgui_modifier(key) {
                            renderer
                                .imgui
                                .io_mut()
                                .add_key_event(key, input.state == ElementState::Pressed);
                        }
                        if let Some(key) = to_imgui_key(key) {
                            renderer
                                .imgui
                                .io_mut()
                                .add_key_event(key, input.state == ElementState::Pressed);
                        }
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    renderer
                        .imgui
                        .io_mut()
                        .add_key_event(imgui::Key::ModShift, modifiers.shift());
                    renderer
                        .imgui
                        .io_mut()
                        .add_key_event(imgui::Key::ModCtrl, modifiers.ctrl());
                    renderer
                        .imgui
                        .io_mut()
                        .add_key_event(imgui::Key::ModAlt, modifiers.alt());
                    renderer
                        .imgui
                        .io_mut()
                        .add_key_event(imgui::Key::ModSuper, modifiers.logo());
                }
                WindowEvent::CursorMoved { position, .. } => {
                    renderer
                        .imgui
                        .io_mut()
                        .add_mouse_pos_event([position.x as f32, position.y as f32]);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if let Some(mouse) = to_imgui_mouse(button) {
                        renderer
                            .imgui
                            .io_mut()
                            .add_mouse_button_event(mouse, state == ElementState::Pressed);
                    }
                }
                WindowEvent::Resized(new_size) => {
                    renderer.recreate_swapchain(new_size);
                    renderer.imgui.io_mut().display_size =
                        [new_size.width as f32, new_size.height as f32];
                }
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            // TODO: Handle key release events outside of the window.
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => input_state.apply_mouse(delta),
                _ => (),
            },
            // This is an indication that it's now allowed to create a graphics context, but the
            // limitation only applies on some platforms (Android).
            Event::Resumed => (),
            Event::MainEventsCleared => {
                let curr_update = Instant::now();
                let delta_time = (curr_update - last_update).as_secs_f32();
                last_update = curr_update;
                world.update(delta_time, &input_state);
                input_state.reset_after_frame();
                if renderer.imgui.io().want_set_mouse_pos {
                    window
                        .window
                        .set_cursor_position(PhysicalPosition {
                            x: renderer.imgui.io().mouse_pos[0],
                            y: renderer.imgui.io().mouse_pos[1],
                        })
                        .unwrap();
                }
                renderer.draw_frame(&world, window.window.inner_size());
            }
            // This event is only sent after MainEventsCleared, during which we render
            // unconditionally.
            Event::RedrawRequested(_) => (),
            // This happens after redraws of all windows are finished, which isn't really applicable
            // to games.
            Event::RedrawEventsCleared => (),
            // Eventually, I should change this from a run_return invocation to normal run, and
            // handle all the Vulkan resource teardown during this event.
            Event::LoopDestroyed => (),
            _ => (),
        }
    });
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
        // VirtualKeyCode:: => Some(imgui::Key::LeftArrow),
        // VirtualKeyCode:: => Some(imgui::Key::RightArrow),
        // VirtualKeyCode:: => Some(imgui::Key::UpArrow),
        // VirtualKeyCode:: => Some(imgui::Key::DownArrow),
        // VirtualKeyCode:: => Some(imgui::Key::PageUp),
        // VirtualKeyCode:: => Some(imgui::Key::PageDown),
        // VirtualKeyCode:: => Some(imgui::Key::Home),
        // VirtualKeyCode:: => Some(imgui::Key::End),
        // VirtualKeyCode:: => Some(imgui::Key::Insert),
        // VirtualKeyCode:: => Some(imgui::Key::Delete),
        // VirtualKeyCode:: => Some(imgui::Key::Backspace),
        // VirtualKeyCode:: => Some(imgui::Key::Space),
        // VirtualKeyCode:: => Some(imgui::Key::Enter),
        // VirtualKeyCode:: => Some(imgui::Key::Escape),
        // VirtualKeyCode:: => Some(imgui::Key::LeftCtrl),
        // VirtualKeyCode:: => Some(imgui::Key::LeftShift),
        // VirtualKeyCode:: => Some(imgui::Key::LeftAlt),
        // VirtualKeyCode:: => Some(imgui::Key::LeftSuper),
        // VirtualKeyCode:: => Some(imgui::Key::RightCtrl),
        // VirtualKeyCode:: => Some(imgui::Key::RightShift),
        // VirtualKeyCode:: => Some(imgui::Key::RightAlt),
        // VirtualKeyCode:: => Some(imgui::Key::RightSuper),
        // VirtualKeyCode:: => Some(imgui::Key::Menu),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha0),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha1),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha2),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha3),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha4),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha5),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha6),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha7),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha8),
        // VirtualKeyCode:: => Some(imgui::Key::Alpha9),
        // VirtualKeyCode:: => Some(imgui::Key::A),
        // VirtualKeyCode:: => Some(imgui::Key::B),
        // VirtualKeyCode:: => Some(imgui::Key::C),
        // VirtualKeyCode:: => Some(imgui::Key::D),
        // VirtualKeyCode:: => Some(imgui::Key::E),
        // VirtualKeyCode:: => Some(imgui::Key::F),
        // VirtualKeyCode:: => Some(imgui::Key::G),
        // VirtualKeyCode:: => Some(imgui::Key::H),
        // VirtualKeyCode:: => Some(imgui::Key::I),
        // VirtualKeyCode:: => Some(imgui::Key::J),
        // VirtualKeyCode:: => Some(imgui::Key::K),
        // VirtualKeyCode:: => Some(imgui::Key::L),
        // VirtualKeyCode:: => Some(imgui::Key::M),
        // VirtualKeyCode:: => Some(imgui::Key::N),
        // VirtualKeyCode:: => Some(imgui::Key::O),
        // VirtualKeyCode:: => Some(imgui::Key::P),
        // VirtualKeyCode:: => Some(imgui::Key::Q),
        // VirtualKeyCode:: => Some(imgui::Key::R),
        // VirtualKeyCode:: => Some(imgui::Key::S),
        // VirtualKeyCode:: => Some(imgui::Key::T),
        // VirtualKeyCode:: => Some(imgui::Key::U),
        // VirtualKeyCode:: => Some(imgui::Key::V),
        // VirtualKeyCode:: => Some(imgui::Key::W),
        // VirtualKeyCode:: => Some(imgui::Key::X),
        // VirtualKeyCode:: => Some(imgui::Key::Y),
        // VirtualKeyCode:: => Some(imgui::Key::Z),
        // VirtualKeyCode:: => Some(imgui::Key::F1),
        // VirtualKeyCode:: => Some(imgui::Key::F2),
        // VirtualKeyCode:: => Some(imgui::Key::F3),
        // VirtualKeyCode:: => Some(imgui::Key::F4),
        // VirtualKeyCode:: => Some(imgui::Key::F5),
        // VirtualKeyCode:: => Some(imgui::Key::F6),
        // VirtualKeyCode:: => Some(imgui::Key::F7),
        // VirtualKeyCode:: => Some(imgui::Key::F8),
        // VirtualKeyCode:: => Some(imgui::Key::F9),
        // VirtualKeyCode:: => Some(imgui::Key::F10),
        // VirtualKeyCode:: => Some(imgui::Key::F11),
        // VirtualKeyCode:: => Some(imgui::Key::F12),
        // VirtualKeyCode:: => Some(imgui::Key::Apostrophe),
        // VirtualKeyCode:: => Some(imgui::Key::Comma),
        // VirtualKeyCode:: => Some(imgui::Key::Minus),
        // VirtualKeyCode:: => Some(imgui::Key::Period),
        // VirtualKeyCode:: => Some(imgui::Key::Slash),
        // VirtualKeyCode:: => Some(imgui::Key::Semicolon),
        // VirtualKeyCode:: => Some(imgui::Key::Equal),
        // VirtualKeyCode:: => Some(imgui::Key::LeftBracket),
        // VirtualKeyCode:: => Some(imgui::Key::Backslash),
        // VirtualKeyCode:: => Some(imgui::Key::RightBracket),
        // VirtualKeyCode:: => Some(imgui::Key::GraveAccent),
        // VirtualKeyCode:: => Some(imgui::Key::CapsLock),
        // VirtualKeyCode:: => Some(imgui::Key::ScrollLock),
        // VirtualKeyCode:: => Some(imgui::Key::NumLock),
        // VirtualKeyCode:: => Some(imgui::Key::PrintScreen),
        // VirtualKeyCode:: => Some(imgui::Key::Pause),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad0),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad1),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad2),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad3),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad4),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad5),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad6),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad7),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad8),
        // VirtualKeyCode:: => Some(imgui::Key::Keypad9),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadDecimal),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadDivide),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadMultiply),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadSubtract),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadAdd),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadEnter),
        // VirtualKeyCode:: => Some(imgui::Key::KeypadEqual),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadStart),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadBack),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadFaceLeft),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadFaceRight),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadFaceUp),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadFaceDown),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadDpadLeft),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadDpadRight),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadDpadUp),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadDpadDown),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadL1),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadR1),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadL2),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadR2),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadL3),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadR3),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadLStickLeft),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadLStickRight),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadLStickUp),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadLStickDown),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadRStickLeft),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadRStickRight),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadRStickUp),
        // VirtualKeyCode:: => Some(imgui::Key::GamepadRStickDown),
        // VirtualKeyCode:: => Some(imgui::Key::MouseLeft),
        // VirtualKeyCode:: => Some(imgui::Key::MouseRight),
        // VirtualKeyCode:: => Some(imgui::Key::MouseMiddle),
        // VirtualKeyCode:: => Some(imgui::Key::MouseX1),
        // VirtualKeyCode:: => Some(imgui::Key::MouseX2),
        // VirtualKeyCode:: => Some(imgui::Key::MouseWheelX),
        // VirtualKeyCode:: => Some(imgui::Key::MouseWheelY),
        // VirtualKeyCode:: => Some(imgui::Key::ReservedForModCtrl),
        // VirtualKeyCode:: => Some(imgui::Key::ReservedForModShift),
        // VirtualKeyCode:: => Some(imgui::Key::ReservedForModAlt),
        // VirtualKeyCode:: => Some(imgui::Key::ReservedForModSuper),
        // VirtualKeyCode:: => Some(imgui::Key::ModCtrl),
        // VirtualKeyCode:: => Some(imgui::Key::ModShift),
        // VirtualKeyCode:: => Some(imgui::Key::ModAlt),
        // VirtualKeyCode:: => Some(imgui::Key::ModSuper),
        // VirtualKeyCode:: => Some(imgui::Key::ModShortcut),
        _ => None,
    }
}

fn to_imgui_mouse(button: MouseButton) -> Option<imgui::MouseButton> {
    // TODO: Convert Other button to Extra1/Extra2?
    match button {
        MouseButton::Left => Some(imgui::MouseButton::Left),
        MouseButton::Right => Some(imgui::MouseButton::Right),
        MouseButton::Middle => Some(imgui::MouseButton::Middle),
        _ => None,
    }
}
