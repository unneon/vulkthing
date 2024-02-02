#![feature(array_chunks)]
#![feature(extract_if)]
#![feature(inline_const)]
#![feature(int_roundings)]
#![feature(iter_array_chunks)]
#![feature(maybe_uninit_write_slice)]
#![feature(panic_info_message)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::manual_map)]
#![allow(clippy::single_match)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod camera;
mod cli;
mod config;
mod grass;
mod input;
mod interface;
mod logger;
mod mesh;
mod physics;
mod planet;
mod renderer;
mod util;
mod voxel;
mod window;
mod world;

use crate::cli::Args;
use crate::config::{
    DEFAULT_GRASS, DEFAULT_PLANET, DEFAULT_RENDERER_SETTINGS, DEFAULT_VOXEL_CHUNK_SIZE,
};
use crate::grass::{GrassResponse, GrassState};
use crate::input::InputState;
use crate::interface::Interface;
use crate::logger::{initialize_logger, initialize_panic_hook};
use crate::mesh::load_mesh;
use crate::planet::generate_planet;
use crate::renderer::Renderer;
use crate::voxel::Voxels;
use crate::window::create_window;
use crate::world::World;
use log::debug;
use nalgebra::Vector3;
use rand::random;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, Event, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;

const VULKAN_APP_NAME: &str = "Vulkthing";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "Unneongine";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

const WALK_SPEED: f32 = 25.;
const SPRINT_SPEED: f32 = 100.;
const CAMERA_SENSITIVITY: f32 = 0.01;

const BENCHMARK_FRAMES: usize = 800;

fn main() {
    initialize_logger();
    initialize_panic_hook();
    let args = Args::parse();
    let window = create_window(&args);
    let cube_mesh = load_mesh("assets/cube.obj");
    let grass_mesh = load_mesh("assets/grass.obj");
    let tetrahedron_mesh = load_mesh("assets/tetrahedron.obj");
    let icosahedron_mesh = load_mesh("assets/icosahedron.obj");
    let mut planet = DEFAULT_PLANET;
    let planet_mesh = Arc::new(generate_planet(&planet));
    let voxels = Voxels::new(DEFAULT_VOXEL_CHUNK_SIZE, random());
    let chunk0 = Vector3::new(0, 0, 0);
    let chunk1 = Vector3::new(1, 0, 0);
    let heightmap0 = voxels.generate_chunk_heightmap(chunk0);
    debug!("voxel heightmap 1 generated");
    let heightmap1 = voxels.generate_chunk_heightmap(chunk1);
    debug!("voxel heightmap 2 generated");
    let svo0 = voxels.generate_chunk_svo(chunk0, &heightmap0);
    debug!("sparse voxel octree 1 generated");
    let svo1 = voxels.generate_chunk_svo(chunk1, &heightmap1);
    debug!("sparse voxel octree 2 generated");
    let triangles0 = voxels.generate_chunk_mesh(&svo0);
    debug!("voxel mesh 1 generated ({})", triangles0.vertices.len());
    let triangles1 = voxels.generate_chunk_mesh(&svo1);
    debug!("voxel mesh 2 generated ({})", triangles1.vertices.len());
    let mut world = World::new(&planet_mesh);
    let mut renderer_settings = DEFAULT_RENDERER_SETTINGS;
    let mut renderer = Renderer::new(
        &window,
        &[
            &planet_mesh,
            &cube_mesh,
            &tetrahedron_mesh,
            &grass_mesh,
            &icosahedron_mesh,
            &triangles0,
            &triangles1,
        ],
        &world,
        &args,
    );
    let mut interface = Interface::new(
        renderer.swapchain.extent.width as usize,
        renderer.swapchain.extent.height as usize,
    );
    let mut grass_parameters = DEFAULT_GRASS;
    let mut grass_state = GrassState::new(&grass_parameters, &planet_mesh, renderer.dev.clone());
    let mut input_state = InputState::new();
    let mut last_update = Instant::now();
    let mut old_size = window.window.inner_size();
    let mut frame_index = 0;

    renderer.create_interface_renderer(&mut interface.ctx);

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, always rendering after
    // MainEventsCleared is enough.
    let loop_result = window.event_loop.run(|event, target| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                // winit is set up for desktop applications by default, so we need to enable polling
                // regardless of whether there are any new events.
                target.set_control_flow(ControlFlow::Poll);
            }
            Event::WindowEvent { event, .. } => {
                interface.apply_window(&event);
                match event {
                    WindowEvent::KeyboardInput { event, .. } => input_state.apply_keyboard(event),
                    WindowEvent::Resized(new_size) => {
                        // On app launch under GNOME/Wayland, winit will send a resize event even if
                        // the size happens to be the same (the focus status also seems to change).
                        // Let's avoid rebuilding the pipelines in this case.
                        if new_size != old_size {
                            debug!(
                                "window resized from {}x{} to {}x{}",
                                old_size.width, old_size.height, new_size.width, new_size.height
                            );
                            renderer.recreate_swapchain(new_size);
                            old_size = new_size;
                        }
                    }
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _ => (),
                }
            }
            // TODO: Handle key release events outside of the window.
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => input_state.apply_mouse(delta),
                _ => (),
            },
            // Desktop applications shouldn't render here according to winit documentation, but this
            // is a game so it's necessary for the game to render even if the camera is not moving.
            // Though I think this approach actually has a problem with input lag. The renderer has
            // to wait on Vulkan fences internally, so rather, this waiting should be done in a
            // background thread and notifications integrated into winit's event loop?
            Event::AboutToWait => {
                let curr_update = Instant::now();
                let delta_time = if args.benchmark {
                    0.01
                } else {
                    (curr_update - last_update).as_secs_f32()
                };
                last_update = curr_update;
                if args.benchmark {
                    world.update_benchmark(frame_index);
                }
                world.update(delta_time, &input_state, args.benchmark);
                input_state.reset_after_frame();
                interface.apply_cursor(input_state.camera_lock, &window.window);
                let interface_events = interface.build(
                    &mut world,
                    &mut planet,
                    &mut grass_parameters,
                    &mut renderer_settings,
                    renderer.pass_times.as_ref(),
                );
                assert!(!interface_events.planet_changed);
                if interface_events.rebuild_swapchain {
                    renderer.recreate_swapchain(window.window.inner_size());
                } else if interface_events.rebuild_pipelines {
                    renderer.recreate_pipelines();
                }

                if interface_events.grass_changed {
                    grass_state.update_parameters(&grass_parameters);
                }
                grass_state.update_camera(world.camera.position());
                for grass_event in grass_state.events() {
                    match grass_event {
                        GrassResponse::Load(chunk_id, chunk) => {
                            renderer.grass_load_chunk(chunk_id, chunk)
                        }
                        GrassResponse::Unload(chunk_id) => renderer.grass_unload_chunk(chunk_id),
                    }
                }

                renderer.draw_frame(
                    &world,
                    &grass_parameters,
                    &renderer_settings,
                    window.window.inner_size(),
                    interface.draw_data(),
                );

                if renderer.just_completed_first_render {
                    window.window.set_visible(true);
                }

                frame_index += 1;
                if args.benchmark && frame_index == BENCHMARK_FRAMES {
                    target.exit();
                }
            }
            // TODO: Handle all the Vulkan resource teardown during this event.
            Event::LoopExiting => (),
            _ => (),
        }
    });
    loop_result.unwrap();
    grass_state.shutdown();
}
