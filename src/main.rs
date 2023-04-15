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
use crate::window::{create_window, to_imgui_key, to_imgui_modifier, to_imgui_mouse};
use crate::world::World;
use std::time::Instant;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, ElementState, Event, StartCause, WindowEvent};

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
                renderer.draw_frame(&mut world, window.window.inner_size());
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
