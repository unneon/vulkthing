#![feature(const_cstr_methods)]

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::{vk, Entry};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::CStr;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;
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
    let mut event_loop = EventLoop::new();
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
    let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
        .unwrap()
        .as_ptr()];
    let mut extension_names =
        ash_window::enumerate_required_extensions(window.raw_display_handle())
            .unwrap()
            .to_vec();
    extension_names.push(
        CStr::from_bytes_with_nul(b"VK_EXT_debug_utils\0")
            .unwrap()
            .as_ptr(),
    );
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layer_names)
        .enabled_extension_names(&extension_names);
    let instance = unsafe { entry.create_instance(&instance_create_info, None) }.unwrap();

    // Set up the callback for debug messages from validation layers. vulkan-tutorial.com also shows
    // how to enable this for creating instances, but ash example doesn't include this.
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));
    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_call_back =
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap();

    // Create the KHR extension surface from winit object. This must be done before selecting a
    // physical device to check whether is supports presenting to the display.
    let surface = unsafe {
        ash_window::create_surface(
            &entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
    }
    .unwrap();
    let surface_loader = Surface::new(&entry, &instance);

    // Select the GPU. For now, just select the first discrete GPU with graphics support. Later,
    // this should react better to iGPU, dGPU and iGPU+dGPU setups. In more complex setups, it would
    // be neat if you could start the game on any GPU, display a choice to the user and seamlessly
    // switch to a new physical device.
    let mut found = None;
    for device in unsafe { instance.enumerate_physical_devices() }.unwrap() {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let gpu_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_str()
            .unwrap();
        let queues = unsafe { instance.get_physical_device_queue_family_properties(device) };
        let is_discrete_gpu = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
        let has_graphics_queue = queues
            .iter()
            .any(|queue| queue.queue_flags.contains(vk::QueueFlags::GRAPHICS));
        let has_present_queue = (0..queues.len() as u32).into_iter().any(|i| {
            unsafe { surface_loader.get_physical_device_surface_support(device, i, surface) }
                .unwrap()
        });
        dbg!(
            gpu_name,
            is_discrete_gpu,
            has_graphics_queue,
            has_present_queue
        );
        if is_discrete_gpu && has_graphics_queue && has_present_queue && found.is_none() {
            println!("selected gpu: {gpu_name}");
            found = Some((device, queues));
        }
    }
    let Some((physical_device, queues)) = found else {
        panic!("gpu not found");
    };

    // Specify physical device extensions, required queues and create them. Probably should pick
    // queues more reasonably?
    let graphics_queue_index = queues
        .iter()
        .enumerate()
        .find(|(_, queue)| queue.queue_flags.contains(vk::QueueFlags::GRAPHICS))
        .unwrap()
        .0 as u32;
    let present_queue_index = (0..queues.len() as u32)
        .into_iter()
        .find(|i| {
            unsafe {
                surface_loader.get_physical_device_surface_support(physical_device, *i, surface)
            }
            .unwrap()
        })
        .unwrap();
    let queue_indices = HashSet::from([graphics_queue_index, present_queue_index]);
    let queue_creates: Vec<_> = queue_indices
        .iter()
        .map(|queue_index| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*queue_index)
                .queue_priorities(&[1.])
                .build()
        })
        .collect();
    let physical_device_features = vk::PhysicalDeviceFeatures::builder();
    let device = unsafe {
        instance.create_device(
            physical_device,
            &vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_creates)
                .enabled_features(&physical_device_features)
                .enabled_layer_names(&layer_names),
            None,
        )
    }
    .unwrap();
    let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
    let present_queue = unsafe { device.get_device_queue(present_queue_index, 0) };

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, initially I'll render at both
    // events, but this probably needs to be changed to alter framebuffer size if the window is
    // resized?
    event_loop.run_return(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => control_flow.set_exit(),
            Event::MainEventsCleared | Event::RedrawRequested(_) => {
                // render
                control_flow.set_exit();
            }
            _ => (),
        }
    });

    unsafe { device.destroy_device(None) };
    unsafe { surface_loader.destroy_surface(surface, None) };
    unsafe { debug_utils_loader.destroy_debug_utils_messenger(debug_call_back, None) };
    unsafe { instance.destroy_instance(None) };
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );
    vk::FALSE
}
