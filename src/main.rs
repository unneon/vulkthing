#![feature(const_cstr_methods)]

use ash::{vk, Entry};
use raw_window_handle::HasRawDisplayHandle;
use std::ffi::CStr;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

const WINDOW_TITLE: &str = "Vulkthing";
const WINDOW_SIZE: (usize, usize) = (800, 600);

const VULKAN_APP_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Vulkthing\0") };
const VULKAN_APP_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);
const VULKAN_ENGINE_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Vulkthing\0") };
const VULKAN_ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

fn main() {
    // Create the application window using winit. Use a predefined size for now, though games should
    // run in fullscreen eventually.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(
            WINDOW_SIZE.0 as f64,
            WINDOW_SIZE.1 as f64,
        ))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    // Load the Vulkan library, and check what extensions it implements. This should probably use
    // the dynamically loaded variant instead?
    let entry = unsafe { Entry::load() }.unwrap();
    println!("scanning for vulkan extensions:");
    for ext in entry.enumerate_instance_extension_properties(None).unwrap() {
        println!("  {ext:?}");
    }

    // Create a Vulkan instance. Mostly metadata of the app and the engine, as well as required
    // extensions.
    let app_info = vk::ApplicationInfo::builder()
        .application_name(VULKAN_APP_NAME)
        .application_version(VULKAN_APP_VERSION)
        .engine_name(VULKAN_ENGINE_NAME)
        .engine_version(VULKAN_ENGINE_VERSION)
        .api_version(vk::API_VERSION_1_0);
    let extension_names = ash_window::enumerate_required_extensions(window.raw_display_handle())
        .unwrap()
        .to_vec();
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&[])
        .enabled_extension_names(&extension_names);
    let instance = unsafe { entry.create_instance(&instance_create_info, None) }.unwrap();

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, initially I'll render at both
    // events, but this probably needs to be changed to alter framebuffer size if the window is
    // resized?
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                unsafe { instance.destroy_instance(None) };
                control_flow.set_exit()
            }
            Event::MainEventsCleared | Event::RedrawRequested(_) => {
                // render
            }
            _ => (),
        }
    });
}
