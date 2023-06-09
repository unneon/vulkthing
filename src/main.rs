#![feature(array_chunks)]
#![feature(drain_filter)]
#![feature(int_roundings)]
#![feature(maybe_uninit_write_slice)]
#![feature(option_as_slice)]
#![feature(pointer_byte_offsets)]
#![feature(inline_const)]
#![feature(iter_array_chunks)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::manual_map)]
#![allow(clippy::single_match)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod camera;
mod config;
mod grass;
mod input;
mod interface;
mod logger;
mod model;
mod physics;
mod planet;
mod renderer;
mod window;
mod world;

use crate::config::{
    DEFAULT_FRAG_SETTINGS, DEFAULT_GRASS, DEFAULT_PLANET, DEFAULT_POSTPROCESSING,
    DEFAULT_RENDERER_SETTINGS,
};
use crate::grass::generate_grass_blades;
use crate::input::InputState;
use crate::interface::Interface;
use crate::logger::initialize_logger;
use crate::model::load_model;
use crate::planet::generate_planet;
use crate::renderer::Renderer;
use crate::window::create_window;
use crate::world::World;
use std::collections::HashSet;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;
use winit::event::{DeviceEvent, Event, StartCause, WindowEvent};

const VULKAN_APP_NAME: &str = "Vulkthing";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "Unneongine";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

const WALK_SPEED: f32 = 25.;
const SPRINT_SPEED: f32 = 100.;
const CAMERA_SENSITIVITY: f32 = 0.01;

fn main() {
    initialize_logger();
    let window = create_window();
    let cube_model = load_model("assets/cube.obj");
    let grass_model = load_model("assets/grass.obj");
    let mut planet = DEFAULT_PLANET;
    let grass = Arc::new(Mutex::new(DEFAULT_GRASS));
    let planet_model = Arc::new(generate_planet(&planet));
    let chunks: Arc<Vec<Vec<usize>>> = Arc::new(grass::build_triangle_chunks(
        &grass.lock().unwrap(),
        &planet,
        &planet_model,
    ));
    let mut renderer = Renderer::new(&window, &[&planet_model, &cube_model], &grass_model);
    let mut interface = Interface::new(
        renderer.swapchain.extent.width as usize,
        renderer.swapchain.extent.height as usize,
    );
    let mut input_state = InputState::new();
    let mut world = World::new(&planet_model);
    let mut last_update = Instant::now();
    let mut renderer_settings = DEFAULT_RENDERER_SETTINGS;
    let mut frag_settings = DEFAULT_FRAG_SETTINGS;
    let mut postprocessing = DEFAULT_POSTPROCESSING;
    let mut old_size = window.window.inner_size();
    let mut loaded_chunks = HashSet::new();

    renderer.create_interface_renderer(&mut interface.ctx);

    let (chunk_tx, chunk_rx) = mpsc::channel::<usize>();
    let async_loader = renderer.get_async_loader();
    std::thread::spawn({
        let chunks = chunks.clone();
        let grass = grass.clone();
        let planet_model = planet_model.clone();
        move || {
            while let Ok(chunk_id) = chunk_rx.recv() {
                let chunk: &[usize] = chunks[chunk_id].as_slice();
                let grass = grass.lock().unwrap().clone();
                let blades = generate_grass_blades(&grass, &planet_model, chunk);
                async_loader.load_grass_chunk(chunk_id, &blades);
            }
        }
    });

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
            Event::WindowEvent { event, .. } => {
                interface.apply_window(&event);
                match event {
                    WindowEvent::KeyboardInput { input, .. } => input_state.apply_keyboard(input),
                    WindowEvent::Resized(new_size) => {
                        // On app launch under GNOME/Wayland, winit will send a resize event even if
                        // the size happens to be the same (the focus status also seems to change).
                        // Let's avoid rebuilding the pipelines in this case.
                        if new_size != old_size {
                            renderer.recreate_swapchain(new_size);
                            old_size = new_size;
                        }
                    }
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    _ => (),
                }
            }
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
                interface.apply_cursor(input_state.camera_lock, &window.window);
                let interface_events = interface.build(
                    &mut planet,
                    &mut grass.lock().unwrap(),
                    &mut renderer_settings,
                    &mut frag_settings,
                    &mut postprocessing,
                    &renderer,
                );
                assert!(!interface_events.planet_changed);
                renderer.unload_grass_chunks(
                    |chunk_id| {
                        let triangle_id = chunks[chunk_id][0];
                        let vertex = planet_model.vertices[3 * triangle_id];
                        (vertex.position - world.camera.position()).norm()
                            > grass.lock().unwrap().chunk_unload_distance
                    },
                    |chunk_id| {
                        loaded_chunks.remove(&chunk_id);
                    },
                );
                for (chunk_id, chunk) in chunks.iter().enumerate() {
                    if !loaded_chunks.contains(&chunk_id) {
                        let triangle_id = chunk[0];
                        let vertex = planet_model.vertices[3 * triangle_id];
                        let distance = (vertex.position - world.camera.position()).norm();
                        if distance < grass.lock().unwrap().chunk_load_distance {
                            loaded_chunks.insert(chunk_id);
                            chunk_tx.send(chunk_id).unwrap();
                        }
                    }
                }

                let grass = grass.lock().unwrap().clone();
                renderer.draw_frame(
                    &world,
                    &grass,
                    &renderer_settings,
                    &frag_settings,
                    &postprocessing,
                    window.window.inner_size(),
                    interface.draw_data(),
                );
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
