#![feature(array_chunks)]
#![feature(extract_if)]
#![feature(inline_const)]
#![feature(int_roundings)]
#![feature(iter_array_chunks)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
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

use crate::cli::Args;
use crate::config::DEFAULT_RENDERER_SETTINGS;
use crate::input::InputState;
use crate::interface::Interface;
use crate::logger::{initialize_logger, initialize_panic_hook};
use crate::mesh::load_mesh;
use crate::renderer::Renderer;
use crate::voxels::Voxels;
use crate::window::create_window;
use crate::world::World;
use log::debug;
use rand::random;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use winit::event::{DeviceEvent, Event, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;

mod camera;
mod cli;
mod config;
mod input;
mod interface;
mod logger;
mod mesh;
mod physics;
mod renderer;
mod util;
pub mod voxels;
mod window;
mod world;

const VULKAN_APP_NAME: &str = "Vulkthing";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "Unneongine";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

const WALK_SPEED: f32 = 25.;
const SPRINT_SPEED: f32 = 100.;
const CAMERA_SENSITIVITY: f32 = 0.01;

const BENCHMARK_FRAMES: usize = 800;

pub fn main() {
    initialize_logger();
    initialize_panic_hook();
    let args = Args::parse();
    let window = create_window(&args);
    let tetrahedron_mesh = load_mesh("assets/tetrahedron.obj");
    let icosahedron_mesh = load_mesh("assets/icosahedron.obj");
    let mut world = World::new();
    let mut renderer_settings = DEFAULT_RENDERER_SETTINGS;
    let mut renderer = Renderer::new(
        &window,
        &[&tetrahedron_mesh, &icosahedron_mesh],
        &world,
        &args,
    );
    let mut interface = Interface::new(
        renderer.swapchain.extent.width as usize,
        renderer.swapchain.extent.height as usize,
    );
    let mut input_state = InputState::new();
    let mut last_update = Instant::now();
    let mut old_size = window.window.inner_size();
    let mut frame_index = 0;

    renderer.create_interface_renderer(&mut interface.ctx);

    let (mut voxels, voxel_config_tx) = Voxels::new(
        random(),
        renderer.voxel_vertex_buffer.mapped_memory(),
        renderer.voxel_index_buffer.mapped_memory(),
        renderer.voxel_meshlet_buffer.mapped_memory(),
    );
    let mut voxel_config = voxels.config().clone();
    let voxels_camera = Arc::new(Mutex::new(world.camera.position()));
    let voxels_condvar = Arc::new(Condvar::new());
    let voxels_shutdown = voxels.shutdown();
    let voxels_shared_vertex_count = voxels.shared_vertex_count();
    let voxels_shared_index_count = voxels.shared_index_count();
    let voxels_shared_meshlet_count = voxels.shared_meshlet_count();
    let voxels_thread = std::thread::spawn({
        let camera = voxels_camera.clone();
        let condvar = voxels_condvar.clone();
        move || {
            let mut camera_lock = camera.lock().unwrap();
            while !voxels.shutdown().load(Ordering::SeqCst) {
                let camera_position = *camera_lock;
                drop(camera_lock);
                voxels.update_camera(camera_position);
                if voxels.shutdown().load(Ordering::SeqCst) {
                    break;
                }
                camera_lock = if !voxels.config_changed() {
                    condvar.wait(camera.lock().unwrap()).unwrap()
                } else {
                    camera.lock().unwrap()
                };
            }
        }
    });
    renderer.voxels_shared_vertex_count = Some(voxels_shared_vertex_count);
    renderer.voxels_shared_index_count = Some(voxels_shared_index_count);
    renderer.voxels_shared_meshlet_count = Some(voxels_shared_meshlet_count);

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
                let voxel_vertices = renderer
                    .voxels_shared_vertex_count
                    .as_ref()
                    .map(|count| count.load(Ordering::SeqCst));
                let voxel_indices = renderer
                    .voxels_shared_index_count
                    .as_ref()
                    .map(|count| count.load(Ordering::SeqCst));
                let interface_events = interface.build(
                    &mut world,
                    &mut renderer_settings,
                    &mut voxel_config,
                    renderer.pass_times.as_ref(),
                    voxel_indices.map(|n| n / 3),
                    voxel_indices,
                    voxel_vertices,
                );
                assert!(!interface_events.planet_changed);
                if interface_events.rebuild_swapchain {
                    renderer.recreate_swapchain(window.window.inner_size());
                } else if interface_events.rebuild_pipelines {
                    renderer.recreate_pipelines();
                }
                if interface_events.rebuild_voxels {
                    let _ = voxel_config_tx.send(voxel_config.clone());
                }

                *voxels_camera.lock().unwrap() = world.camera.position();
                voxels_condvar.notify_one();

                renderer.draw_frame(
                    &world,
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
    voxels_shutdown.store(true, Ordering::SeqCst);
    voxels_condvar.notify_one();
    voxels_thread.join().unwrap();
    loop_result.unwrap();
}
