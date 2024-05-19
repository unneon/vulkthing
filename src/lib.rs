#![allow(clippy::too_many_arguments)]

use crate::cli::Args;
use crate::config::{DEFAULT_RENDERER_SETTINGS, DEFAULT_VOXEL_CONFIG};
use crate::input::InputState;
#[cfg(feature = "dev-menu")]
use crate::interface::Interface;
use crate::logger::{initialize_logger, initialize_panic_hook};
use crate::mesh::load_mesh;
use crate::renderer::{Renderer, RendererSettings};
use crate::voxel::{Voxels, VoxelsConfig};
use crate::window::create_window;
use crate::world::World;
use log::debug;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

mod camera;
mod cli;
mod config;
mod input;
#[cfg(feature = "dev-menu")]
mod interface;
mod logger;
mod mesh;
mod physics;
mod renderer;
mod util;
pub mod voxel;
mod window;
mod world;

const VULKAN_APP_NAME: &str = "Vulkthing";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "Unneongine";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

const WALK_SPEED: f32 = 25.;
const SPRINT_SPEED: f32 = 100.;
const CAMERA_SENSITIVITY: f32 = 0.01;

struct AppState {
    window: winit::window::Window,
    world: World,
    voxels: Voxels,
    voxels_config: VoxelsConfig,
    renderer: Renderer,
    renderer_settings: RendererSettings,
    input_state: InputState,
    last_window_size: PhysicalSize<u32>,
    last_frame_timestamp: Instant,
    frame_index: usize,
}

impl ApplicationHandler for AppState {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            // winit is set up for desktop applications by default, so we need to enable polling
            // regardless of whether there are any new events.
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }

    fn resumed(&mut self, _: &ActiveEventLoop) {
        // TODO: Create window here instead for correct behavior on Android I think?
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        #[cfg(feature = "dev-menu")]
        interface.apply_window(&event);
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.input_state.apply_keyboard(event),
            WindowEvent::Resized(new_size) => {
                // On app launch under GNOME/Wayland, winit will send a resize event even if
                // the size happens to be the same (the focus status also seems to change).
                // Let's avoid rebuilding the pipelines in this case.
                if new_size != self.last_window_size {
                    let old_size = self.last_window_size;
                    debug!(
                        "window resized from {}x{} to {}x{}",
                        old_size.width, old_size.height, new_size.width, new_size.height
                    );
                    self.renderer.recreate_swapchain(new_size);
                    self.last_window_size = new_size;
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        // TODO: Handle key release events outside of the window.
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input_state.apply_mouse(delta);
        }
    }

    // Desktop applications shouldn't render here according to winit documentation, but this
    // is a game so it's necessary for the game to render even if the camera is not moving.
    // Though I think this approach actually has a problem with input lag. The renderer has
    // to wait on Vulkan fences internally, so rather, this waiting should be done in a
    // background thread and notifications integrated into winit's event loop?
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let current_frame_timestamp = Instant::now();
        let delta_time = (current_frame_timestamp - self.last_frame_timestamp).as_secs_f32();
        self.last_frame_timestamp = current_frame_timestamp;
        self.world.update(delta_time, &self.input_state);
        self.voxels.update_camera(self.world.camera.position());

        self.input_state.reset_after_frame();
        #[cfg(feature = "dev-menu")]
        {
            interface.apply_cursor(input_state.camera_lock, &window.window);
            let interface_events = interface.build(
                &mut world,
                &mut renderer_settings,
                &mut voxel_config,
                renderer.frametime,
            );
            assert!(!interface_events.planet_changed);
            if interface_events.rebuild_swapchain {
                renderer.recreate_swapchain(window.window.inner_size());
            } else if interface_events.rebuild_pipelines {
                renderer.recreate_pipelines();
            }
            if interface_events.rebuild_voxels {
                voxels.update_config(voxel_config.clone());
            }
        }

        self.renderer.draw_frame(
            &self.world,
            &self.voxels_config,
            &self.renderer_settings,
            self.window.inner_size(),
            #[cfg(feature = "dev-menu")]
            interface.draw_data(),
        );

        if self.renderer.just_completed_first_render {
            self.window.set_visible(true);
        }

        self.frame_index += 1;
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        // TODO: Handle all the Vulkan resource teardown during this event.
    }
}

pub fn main() {
    initialize_logger();
    initialize_panic_hook();
    let args = Args::parse();
    let window = create_window(&args);
    let tetrahedron_mesh = load_mesh("assets/tetrahedron.obj");
    let icosahedron_mesh = load_mesh("assets/icosahedron.obj");
    let world = World::new();
    let mut renderer = Renderer::new(
        &window,
        &[&tetrahedron_mesh, &icosahedron_mesh],
        &world,
        &args,
    );
    #[cfg(feature = "dev-menu")]
    let mut interface = Interface::new(
        renderer.swapchain.extent.width as usize,
        renderer.swapchain.extent.height as usize,
    );

    #[cfg(feature = "dev-menu")]
    renderer.create_interface_renderer(&mut interface.ctx);

    let voxels_config = DEFAULT_VOXEL_CONFIG;
    let voxels = Voxels::new(
        voxels_config.clone(),
        world.camera.position(),
        renderer.voxel_gpu_memory.take().unwrap(),
        std::thread::available_parallelism().unwrap().get() - 1,
    );

    let last_window_size = window.window.inner_size();
    let mut app_state = AppState {
        window: window.window,
        world,
        voxels,
        input_state: InputState::new(),
        last_window_size,
        last_frame_timestamp: Instant::now(),
        renderer,
        voxels_config,
        renderer_settings: DEFAULT_RENDERER_SETTINGS,
        frame_index: 0,
    };
    let loop_result = window.event_loop.run_app(&mut app_state);
    app_state.renderer.wait_idle();
    app_state.voxels.shutdown();
    loop_result.unwrap();
}
