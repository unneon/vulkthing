#![allow(clippy::too_many_arguments)]

use crate::cli::{Args, WindowProtocol};
use crate::config::{DEFAULT_RENDERER_SETTINGS, DEFAULT_VOXEL_CONFIG};
use crate::input::InputState;
#[cfg(feature = "dev-menu")]
use crate::interface::Interface;
use crate::logger::{initialize_logger, initialize_panic_hook};
use crate::mesh::load_mesh;
use crate::renderer::{Renderer, RendererSettings};
use crate::voxel::{Voxels, VoxelsConfig};
use crate::world::World;
use log::{debug, warn};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::wayland::EventLoopBuilderExtWayland;
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

mod camera;
mod cli;
mod config;
mod gpu;
mod input;
#[cfg(feature = "dev-menu")]
mod interface;
mod logger;
mod mesh;
mod physics;
mod renderer;
mod util;
pub mod voxel;
mod world;

const WINDOW_TITLE: &str = "Vulkthing";

const VULKAN_APP_NAME: &str = "Vulkthing";
const VULKAN_APP_VERSION: (u32, u32, u32) = (0, 0, 0);
const VULKAN_ENGINE_NAME: &str = "Unneongine";
const VULKAN_ENGINE_VERSION: (u32, u32, u32) = (0, 0, 0);

const WALK_SPEED: f32 = 25.;
const SPRINT_SPEED: f32 = 100.;
const CAMERA_SENSITIVITY: f32 = 0.01;

struct AppState {
    window: Option<Window>,
    world: World,
    // This depends on the lifetime of Renderer, but there isn't a good way to represent this in
    // Rust and I actually had a segfault because of this. Do I have to go with self-referential
    // structs here? Or do I need to fold everything using Vulkan resources into the renderer
    // struct? Thinking about it, it's probably Arc, because there are worker threads involved.
    voxels: Option<Voxels>,
    voxels_config: VoxelsConfig,
    renderer: Option<Renderer>,
    renderer_settings: RendererSettings,
    input_state: InputState,
    #[cfg(feature = "dev-menu")]
    interface: Option<Interface>,
    last_window_size: Option<PhysicalSize<u32>>,
    last_frame_timestamp: Instant,
    frame_index: usize,
    args: Args,
}

impl ApplicationHandler for AppState {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            // winit is set up for desktop applications by default, so we need to enable polling
            // regardless of whether there are any new events.
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_resizable(true)
            .with_decorations(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .with_visible(false);
        let window = event_loop.create_window(window_attributes).unwrap();
        if window.set_cursor_grab(CursorGrabMode::Locked).is_err() {
            warn!("cursor grab unavailable");
        }
        window.set_cursor_visible(false);

        let tetrahedron_mesh = load_mesh("assets/tetrahedron.obj");
        let icosahedron_mesh = load_mesh("assets/icosahedron.obj");
        let mut renderer = Renderer::new(
            &window,
            &[&tetrahedron_mesh, &icosahedron_mesh],
            &self.world,
            &self.args,
        );

        #[cfg(feature = "dev-menu")]
        {
            let mut interface = Interface::new(
                renderer.swapchain.extent.width as usize,
                renderer.swapchain.extent.height as usize,
            );
            renderer.create_interface_renderer(&mut interface.ctx);
            self.interface = Some(interface);
        }

        let voxels = Voxels::new(
            self.voxels_config.clone(),
            self.world.camera.position(),
            renderer.voxel_gpu_memory.take().unwrap(),
            std::thread::available_parallelism().unwrap().get() - 1,
        );

        self.last_window_size = Some(window.inner_size());
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.voxels = Some(voxels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        #[cfg(feature = "dev-menu")]
        self.interface.as_mut().unwrap().apply_window(&event);
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.input_state.apply_keyboard(event),
            WindowEvent::Resized(new_size) => {
                // On app launch under GNOME/Wayland, winit will send a resize event even if
                // the size happens to be the same (the focus status also seems to change).
                // Let's avoid rebuilding the pipelines in this case.
                if Some(new_size) != self.last_window_size {
                    if let Some(old_size) = self.last_window_size {
                        debug!(
                            "window resized from {}x{} to {}x{}",
                            old_size.width, old_size.height, new_size.width, new_size.height
                        );
                    } else {
                        debug!(
                            "window initially resized to {}x{}",
                            new_size.width, new_size.height
                        );
                    }
                    self.renderer.as_mut().unwrap().recreate_swapchain(new_size);
                    self.last_window_size = Some(new_size);
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
        self.voxels
            .as_mut()
            .unwrap()
            .update_camera(self.world.camera.position());

        self.input_state.reset_after_frame();
        #[cfg(feature = "dev-menu")]
        {
            self.interface
                .as_mut()
                .unwrap()
                .apply_cursor(self.input_state.camera_lock, self.window.as_ref().unwrap());
            let interface_events = self.interface.as_mut().unwrap().build(
                &mut self.world,
                &mut self.renderer_settings,
                &mut self.voxels_config,
                self.renderer.as_ref().unwrap().frametime,
            );
            assert!(!interface_events.planet_changed);
            if interface_events.rebuild_swapchain {
                self.renderer
                    .as_mut()
                    .unwrap()
                    .recreate_swapchain(self.window.as_ref().unwrap().inner_size());
            } else if interface_events.rebuild_pipelines {
                self.renderer.as_mut().unwrap().recreate_pipelines();
            }
            if interface_events.rebuild_voxels {
                self.voxels
                    .as_mut()
                    .unwrap()
                    .update_config(self.voxels_config.clone());
            }
        }

        self.renderer.as_mut().unwrap().draw_frame(
            &self.world,
            &self.voxels_config,
            &self.renderer_settings,
            self.window.as_ref().unwrap().inner_size(),
            #[cfg(feature = "dev-menu")]
            self.interface.as_mut().unwrap().draw_data(),
        );

        if self.renderer.as_ref().unwrap().just_completed_first_render {
            self.window.as_mut().unwrap().set_visible(true);
        }

        self.frame_index += 1;
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        if let Some(renderer) = self.renderer.take() {
            renderer.wait_idle();
            self.voxels.take().unwrap().shutdown();
        }
    }
}

pub fn main() {
    initialize_logger();
    initialize_panic_hook();
    let args = Args::parse();
    let event_loop = create_event_loop(&args);

    let mut app_state = AppState {
        window: None,
        world: World::new(),
        voxels: None,
        voxels_config: DEFAULT_VOXEL_CONFIG,
        input_state: InputState::new(),
        last_window_size: None,
        last_frame_timestamp: Instant::now(),
        renderer: None,
        renderer_settings: DEFAULT_RENDERER_SETTINGS,
        #[cfg(feature = "dev-menu")]
        interface: None,
        frame_index: 0,
        args,
    };
    event_loop.run_app(&mut app_state).unwrap();
}

fn create_event_loop(args: &Args) -> EventLoop<()> {
    let mut event_loop = EventLoop::builder();
    match args.window_protocol {
        Some(WindowProtocol::Wayland) => event_loop.with_wayland(),
        Some(WindowProtocol::X11) => event_loop.with_x11(),
        None => &mut event_loop,
    };
    event_loop.build().unwrap()
}
